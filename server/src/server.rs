use crate::file_loader::load_contents;
use crate::logger::{Logger, Logging};
use mqtt_packet::mqtt_packet_service::header_packet::control_flags::{self};
use mqtt_packet::mqtt_packet_service::header_packet::{control_type, PacketHeader};
use mqtt_packet::mqtt_packet_service::payload_packet::{
    suback_return_codes, Payload, PublishPayload, SubscribePayload, UnsubscribePayload,
};
use mqtt_packet::mqtt_packet_service::variable_header_packet::{
    connect_ack_flags, connect_return, PacketVariableHeader, VariableHeader,
    VariableHeaderPacketIdentifier, VariableHeaderPublish,
};
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet, ServerPacket, Utils};
use rand::Rng;
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::thread::{self};
use std::time::Duration;
type LastWill = (String, String);
type HashPersistanceConnections = HashMap<String, (JoinHandle<()>, String)>; // la clave es el ip address contiene como valor (Joinhandle del thread, el client_id)
type HashServerConnections = HashMap<String, (HandleClientConnections, LastWill)>; // la clave es el client_id de mqtt
type HashTopics = HashMap<String, (Vec<(String, Sender<Vec<u8>>)>, String)>; // tuple value (vec of (client_id, tx senders), message of retain)
type HashCredentials = HashMap<String, String>;
#[derive(Clone)]
pub struct Server {
    server_address: Arc<String>,
    server_port: Arc<String>,
    logger: Arc<Logger>,
    hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>,
    hash_server_connections: Arc<Mutex<HashServerConnections>>,
    hash_topics: Arc<Mutex<HashTopics>>,
    tx_server: Arc<Mutex<Sender<Vec<String>>>>,
    rx_server: Arc<Mutex<Receiver<Vec<String>>>>,
    hash_credentials: Arc<Mutex<HashCredentials>>,
}
#[derive(Clone, Debug)]
pub struct HandleClientConnections {
    tx: Arc<Mutex<Sender<Vec<u8>>>>,
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    #[allow(dead_code)]
    peer: Arc<Mutex<String>>,
}

#[allow(clippy::unit_arg)]
impl Server {
    pub fn new(
        server_address: String,
        server_port: String,
        file_source: &str,
        credentials_file: &str,
    ) -> Server {
        let hash_credentials = if !credentials_file.is_empty() {
            load_contents(credentials_file)
        } else {
            HashMap::new()
        };

        let hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>> =
            Arc::new(Mutex::new(HashMap::new()));
        let hash_server_connections: Arc<Mutex<HashServerConnections>> =
            Arc::new(Mutex::new(HashMap::new()));
        let hash_topics: Arc<Mutex<HashTopics>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx_server, rx_server) = channel::<Vec<String>>();

        let server = Server {
            server_address: Arc::new(server_address),
            server_port: Arc::new(server_port),
            logger: Arc::new(Logger::new(file_source, true)),
            hash_persistance_connections,
            hash_server_connections,
            hash_topics,
            tx_server: Arc::new(Mutex::new(tx_server)),
            rx_server: Arc::new(Mutex::new(rx_server)),
            hash_credentials: Arc::new(Mutex::new(hash_credentials)),
        };
        let _handle = Server::message_handler(
            server.tx_server.clone(),
            server.rx_server.clone(),
            server.hash_topics.clone(),
            server.hash_server_connections.clone(),
            server.logger.clone(),
        );
        server
    }

    fn handle_client(
        peer: String,
        stream: TcpStream,
        logger: Arc<Logger>,
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
        hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>,
        hash_credentials: Arc<Mutex<HashCredentials>>,
        tx_server: Sender<Vec<String>>,
    ) -> Result<JoinHandle<()>> {
        fn _handle_client_(
            mut stream: TcpStream,
            logger: Arc<Logger>,
            hash_server_connections: Arc<Mutex<HashServerConnections>>,
            hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>,
            hash_credentials: Arc<Mutex<HashCredentials>>,
            mut client_connections: HandleClientConnections,
            tx_server: Sender<Vec<String>>,
        ) -> Result<()> {
            let mut buff = [0_u8; 1024];
            let mut _client_id = String::new();
            let keepalive_retry: usize = 300;
            let mut keepalive_count: usize = keepalive_retry;
            #[allow(unreachable_code)]
            Ok(loop {
                // this timeout checks when the client is disconnected
                stream.set_read_timeout(Some(Duration::from_millis(30)))?;
                if let Ok(_size) = stream.read(&mut buff) {
                    if _size > 0 {
                        let control_type = buff[0];
                        logger.debug("Check if a MQTT PACKET is received".to_string());
                        if Packet::<VariableHeader, Payload>::is_mqtt_packet(&buff) {
                            logger.debug(format!("Found a MQTT packet: {:?}", control_type));
                            match Server::handle_packet(
                                buff.to_vec(),
                                &mut stream,
                                logger.clone(),
                                hash_server_connections.clone(),
                                hash_credentials.clone(),
                                &mut client_connections,
                                tx_server.clone(),
                                &mut _client_id,
                            ) {
                                Ok(client_id) => {
                                    logger.debug(format!(
                                        "Packet from peer {} and client id: {} has been processed",
                                        stream.peer_addr()?,
                                        client_id
                                    ));
                                    let peer = client_connections.peer.lock().unwrap().clone();
                                    hash_persistance_connections
                                        .lock()
                                        .unwrap()
                                        .entry(peer.clone())
                                        .and_modify(|e| {
                                            e.1 = client_id.clone();
                                        });

                                    logger.debug("Cleaning buffer".to_string());
                                    buff = [0_u8; 1024];
                                }
                                Err(e) => {
                                    logger.debug(format!("Error (handle_packet): {}", e));
                                    break; // stopping thread
                                }
                            }
                        } else {
                            logger
                                .debug("Clean buffer to continue reading from stream".to_string());
                            buff = [0_u8; 1024];
                        };
                    };
                }

                let client_rx = &*client_connections.rx.lock().unwrap();
                if let Ok(msg) = client_rx.try_recv() {
                    logger.debug("Received message from server through channel".to_string());
                    stream.write_all(&msg)?;
                }

                // server keepalive check using RESERVED bits of mqtt packet
                if keepalive_count == 0 {
                    let msg = vec![0xF0_u8];
                    if let Err(e) = stream.write_all(&msg) {
                        logger.debug(format!("Error (server keepalive failed): {}", e));
                        return Err(e);
                    }
                    keepalive_count = keepalive_retry;
                    logger.debug(format!("Server client id {} keepalive check", _client_id));
                } else {
                    keepalive_count -= 1;
                }

                // ends loop returns ok result
            })
        }

        let (tx, rx) = channel::<Vec<u8>>();

        let handle_client_connections = HandleClientConnections {
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            peer: Arc::new(Mutex::new(peer.to_string())),
        };

        // let peer = stream.peer_addr()?;

        let handle = thread::Builder::new()
            .name("thread peer: ".to_string() + peer.to_string().as_str())
            .spawn(move || {
                // connection succeeded
                logger.debug(format!("Connection from {}", peer));
                match _handle_client_(
                    stream,
                    logger.clone(),
                    hash_server_connections.clone(),
                    hash_persistance_connections.clone(),
                    hash_credentials,
                    handle_client_connections,
                    tx_server.clone(),
                ) {
                    Ok(_) => {
                        logger.debug(format!("Connection with {} closed", peer));
                    }
                    Err(e) => {
                        let client_id = hash_persistance_connections
                            .lock()
                            .unwrap()
                            .get(&peer.to_string())
                            .unwrap()
                            .1
                            .clone();
                        logger.debug(format!(
                            "Error (_handle_client_): {} for client id: {}",
                            e, client_id
                        ));
                        Server::send_last_will(
                            hash_server_connections.clone(),
                            tx_server.clone(),
                            client_id,
                            &logger,
                        );
                        hash_persistance_connections
                            .lock()
                            .unwrap()
                            .remove(&peer.to_string());
                    }
                }
            });
        handle
    }

    pub fn listening(&self) -> Result<()> {
        let server_address = Arc::clone(&self.server_address);
        let server_port = Arc::clone(&self.server_port);
        self.logger.debug("ready to binding".to_string());
        self.logger.info(format!(
            "server address: {:?}",
            server_address.to_string() + ":" + server_port.as_str()
        ));
        let address = format!("{}:{}", server_address, server_port);
        let listener = TcpListener::bind(address)?;
        // accept connections and process them, spawning a new thread for each one
        self.logger.debug("start binding".to_string());
        self.logger
            .debug(format!("Server listening on port {}", self.server_port));

        self.logger
            .info("starting listening to clients".to_string());
        let server_mutex = Arc::new(Mutex::new(self)); // moved self to a Arc Mutex to access the server struct
        for stream in listener.incoming() {
            let _clone_server = Arc::clone(&server_mutex);
            match stream {
                Ok(stream) => {
                    let peer = stream.peer_addr()?;
                    let this = server_mutex.lock().unwrap();
                    let logger = this.logger.clone();
                    logger.info(format!("New client connected: {}", peer));
                    let tx = this.tx_server.lock().unwrap();
                    let _handle = Server::handle_client(
                        peer.to_string(),
                        stream,
                        logger.clone(),
                        this.hash_server_connections.clone(),
                        this.hash_persistance_connections.clone(),
                        this.hash_credentials.clone(),
                        tx.clone(),
                    );
                    if let Err(e) = _handle {
                        logger.error(format!("Error: {}", e));
                        break;
                    }
                    // Updating hash_persistance_connections with the JoinHandle of the clients thread
                    self.hash_persistance_connections
                        .lock()
                        .unwrap()
                        .insert(peer.to_string(), (_handle.unwrap(), "".to_string()));
                }
                Err(e) => {
                    /* connection failed */
                    self.logger.debug(format!("Error: {}", e));
                    break;
                }
            }
        }
        // close the socket server
        drop(listener);
        server_mutex
            .lock()
            .unwrap()
            .logger
            .info("Server terminated.".to_string()); //ver porque no se escribe esta linea no se escribe en el log
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_packet(
        buff: Vec<u8>,
        stream: &mut TcpStream,
        logger: Arc<Logger>,
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
        hash_credentials: Arc<Mutex<HashCredentials>>,
        client_connections: &mut HandleClientConnections,
        tx_server: Sender<Vec<String>>,
        client_id: &mut String,
    ) -> Result<String> {
        let packet_id = buff[0] & 0xF0;

        let peer_addr = stream.peer_addr()?;

        if packet_id == control_type::CONNECT as u8 {
            let unvalued_packet = Packet::<VariableHeader, Payload>::unvalue(buff.clone());
            let client_identifier: String = unvalued_packet.payload.client_identifier;
            *client_id = client_identifier.clone();
            logger.debug(format!(
                "Client Connection with Client identifier: {}, verifying that was connected...",
                client_identifier
            ));

            // Credentials check
            if !hash_credentials.lock().unwrap().is_empty() {
                // get the user and password from the packet
                let user = unvalued_packet.payload.user_name;
                let password = unvalued_packet.payload.password;
                // get user and password from the filename
                if !user.is_empty() || !password.is_empty() {
                    // searching for the user and password in the hashmap and then compare password with value
                    let hash_credentials = hash_credentials.lock().unwrap();
                    match hash_credentials.get(user.as_str()) {
                        Some(password_saved) => {
                            if *password_saved != password {
                                logger.debug(format!("User {} password is incorrect", user));
                                return Err(Error::new(
                                    ErrorKind::Other,
                                    format!("Client Connection with Client identifier: {} refused, user password incorrect", client_identifier),
                                ));
                            }
                        }
                        None => {
                            logger.debug(format!(
                                "User {} not found in server credential file",
                                user
                            ));
                            return Err(Error::new(
                                ErrorKind::Other,
                                format!("Client Connection with Client identifier: {} refused, user not found", client_identifier),
                            ));
                        }
                    }
                } else {
                    logger.debug(format!(
                        "User and password not found in connecting packet for client {}, connecting anyway",
                        client_identifier
                    ));
                }
            } else {
                logger.debug("No credentials registered for this server".to_string());
            }
            // End credentials check

            if Server::get_id_server_connections(client_identifier, hash_server_connections.clone())
                .unwrap()
            {
                logger.debug(format!("Client already connected clientId: {}", client_id));
                // Client Persistance Clean Session, if false must resume communications with the client
                if !unvalued_packet.variable_header.clean_session() {
                    let value = hash_server_connections.lock().unwrap();
                    let old_client_connection = value.get(&client_id.clone());
                    match old_client_connection {
                        Some(old_client_connection) => {
                            // setting old channel to new channel
                            let old_client_connection = &old_client_connection.0;
                            client_connections.tx = old_client_connection.tx.clone();
                            client_connections.rx = old_client_connection.rx.clone();
                            logger.debug(format!(
                                "Client connection set with old channel for clientId: {}",
                                client_id
                            ));
                        }
                        None => {
                            // if none is received there wasnt any connection with this client
                            logger.debug(format!(
                                "Clean session was asked but no connection was found for clientId: {}",
                                client_id
                            ));
                        }
                    };
                } else {
                    // unsubscribe tx of old client client_id from topics
                    match tx_server.send(vec![
                        "request_clean_session".to_string(),
                        client_id.to_string(),
                    ]) {
                        Ok(_) => {
                            logger.debug(format!(
                                "Clean session was requested for clientId: {}",
                                client_id
                            ));
                        }
                        Err(e) => {
                            logger.debug(format!(
                                "Error sending request_clean_session to server: {}",
                                e
                            ));
                        }
                    };
                }

                // do not return error this causes not to send a connack to client on reconnect
                // return Err(Error::new(
                //     ErrorKind::Other,
                //     "Error, client already connected",
                // ));
            }
            // si no lo encuentra debemos seguir sin dar error ...
        }

        // check other packets type
        match packet_id {
            control_type::CONNECT => {
                logger.info("Connect packet received".to_string());
                logger.debug(format!(
                    "Peer mqtt connected: {}, with action type: {} ",
                    peer_addr, packet_id
                ));

                let unvalued_packet = Packet::<VariableHeader, Payload>::unvalue(buff);
                let client_identifier: String = unvalued_packet.payload.client_identifier;
                let will_topic: String = unvalued_packet.payload.will_topic;
                let will_message: String = unvalued_packet.payload.will_message;
                *client_id = client_identifier.clone();

                // check if it has Last will statement and update hash_server_connections
                if will_topic.is_empty() {
                    logger.debug("No Last will statement".to_string());

                    hash_server_connections.lock().unwrap().insert(
                        client_id.to_string(),
                        (client_connections.clone(), ("".to_string(), "".to_string())),
                    );
                } else {
                    logger.debug(format!(
                        "Last will statement topic: {} from client ID: {}",
                        will_topic, client_identifier
                    ));
                    // update last will statement on hash_server_connections
                    hash_server_connections
                        .lock()
                        .unwrap()
                        .entry(client_identifier)
                        .and_modify(|e| {
                            let will_tuple = &mut e.1;
                            will_tuple.0 = will_topic.clone();
                            will_tuple.1 = will_message.clone();
                        })
                        .or_insert((client_connections.clone(), (will_topic, will_message)));
                }

                let packet = Packet::<VariableHeader, Payload>::new();
                let packet =
                    packet.connack(connect_ack_flags::SESSION_PRESENT, connect_return::ACCEPTED);
                logger.debug("Sending connack packet".to_string());
                if let Err(e) = stream.write_all(&packet.value()) {
                    logger.debug("Client disconnect".to_string());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Error: cannot write: {}", e),
                    ));
                }
            }

            control_type::PUBLISH => {
                logger.debug("Publish packet received".to_string());
                logger.debug(format!("Peer mqtt publish: {:?}", peer_addr));

                let unvalue = Packet::<VariableHeaderPublish, PublishPayload>::unvalue(buff);
                let msg_server: Vec<String> = vec![
                    "publish".to_string(),
                    if unvalue.header.get_dup() { 1 } else { 0 }.to_string(),
                    if unvalue.header.get_qos() == control_flags::QOS0 {
                        1
                    } else {
                        0
                    }
                    .to_string(),
                    if unvalue.header.get_retain() { 1 } else { 0 }.to_string(),
                    String::from_utf8_lossy(&unvalue.variable_header.topic_name).to_string(),
                    unvalue.payload.message,
                ];

                match tx_server.send(msg_server) {
                    Ok(_) => {
                        logger.debug(format!(
                            "Message sent to server to process publish for client: {}",
                            client_id
                        ));
                        // send puback if qos is 1 control_flags::QOS0 is when qos is 1
                        if unvalue.header.get_qos() == control_flags::QOS0 {
                            logger.debug("Identified QoS1 flag. PubAck sent".to_string());
                            let packet = Packet::<VariableHeaderPacketIdentifier, Payload>::new();
                            let packet = packet.puback(packet_id as u16);

                            if let Err(e) = stream.write_all(&packet.value()) {
                                logger.debug("Client disconnect".to_string());
                                return Err(Error::new(
                                    ErrorKind::Other,
                                    format!("Error: cannot write: {}", e),
                                ));
                            }
                        } else {
                            logger.debug("Identified QoS0 flag. Nothing to do".to_string());
                        }
                    }
                    Err(e) => {
                        logger.debug(format!("Error sending message to server to process publish from client: {} error: {}", client_id ,e));
                    }
                };
            }

            control_type::DISCONNECT => {
                logger.debug("Disconnect packet received".to_string());
                logger.info(format!("Peer {:?} will be disconnected ", peer_addr));

                match stream.shutdown(Shutdown::Both) {
                    Ok(_) => {
                        logger.debug(format!("Peer {} has been disconnected", peer_addr));
                    }
                    Err(_e) => {}
                }
            }

            control_type::SUBSCRIBE => {
                logger.debug("Suscribe packet received".to_string());
                logger.debug(format!("Peer mqtt suscribe: {:?}", peer_addr));

                let unvalue =
                    Packet::<VariableHeaderPacketIdentifier, SubscribePayload>::unvalue(buff);
                let packet_identifier = unvalue.variable_header.packet_identifier;
                let topics = unvalue.payload.topic_filter;
                let qos_vec = unvalue.payload.qos;

                let mut qos_result = Vec::<u8>::new();
                // enviando al tx del server los topics suscriptos
                logger.debug("Sending tx server to the subcribed topics".to_string());
                for (index, topic) in topics.iter().enumerate() {
                    let msg_server = vec![
                        "subscribe".to_string(),
                        client_id.to_string(),
                        packet_identifier.to_string(),
                        topic.to_string(),
                    ];
                    match tx_server.send(msg_server.clone()) {
                        Ok(_) => {
                            logger.debug(format!("Suscribe topic {} sent to server", topic));
                            qos_result.push(qos_vec[index]);
                        }
                        Err(e) => {
                            logger.debug(format!(
                                "Error sending suscribe topic {} to server: {}",
                                topic, e
                            ));
                            qos_result.push(suback_return_codes::FAILURE);
                        }
                    };
                }
                // enviar el suback
                logger.debug("Sending suback packet to client".to_string());
                let packet = Packet::<VariableHeader, Payload>::new();
                let packet = packet.suback(packet_identifier, qos_result);
                if let Err(e) = stream.write_all(&packet.value()) {
                    logger.debug("Client disconnect".to_string());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Error: cannot write: {}", e),
                    ));
                }
            }

            control_type::UNSUBSCRIBE => {
                logger.debug("Unsubscribe packet received".to_string());
                logger.debug(format!("Peer mqtt unsubscribe: {:?}", peer_addr));

                let unvalue =
                    Packet::<VariableHeaderPacketIdentifier, UnsubscribePayload>::unvalue(buff);
                let packet_identifier = unvalue.variable_header.packet_identifier;
                // need to get the client identifier and the topics to unsubscribe
                let topics = unvalue.payload.topic_filter;
                topics.iter().for_each(|topic| {
                    let msg_server = vec![
                        "unsubscribe".to_string(),
                        client_id.to_string(),
                        packet_identifier.to_string(),
                        topic.to_string(),
                    ];
                    tx_server.send(msg_server.clone()).unwrap_or_else(|_| {
                        panic!("Cannot proccess unsubscribe message {:?}", msg_server)
                    });
                });

                let qos = unvalue.header.get_qos();
                // send the unsuback if the qos is set to 1
                if qos == control_flags::QOS0 {
                    logger.debug(format!(
                        "Sending unsuback packet to client qos set to: {}",
                        qos
                    ));
                    let packet = Packet::<VariableHeader, Payload>::new();
                    let packet = packet.unsuback(packet_identifier);
                    if let Err(e) = stream.write_all(&packet.value()) {
                        logger.debug("Client disconnect".to_string());
                        return Err(Error::new(
                            ErrorKind::Other,
                            format!("Error: cannot write: {}", e),
                        ));
                    }
                }
            }

            control_type::PINGREQ => {
                logger.info("PingReq packet received".to_string());
                logger.debug(format!(
                    "Peer mqtt connected: {}, with action type: {} ",
                    peer_addr, packet_id
                ));
                let packet = Packet::<VariableHeader, Payload>::new();
                let packet = packet.pingresp();
                logger.debug("Sending pingresp packet".to_string());
                if let Err(e) = stream.write_all(&packet.value()) {
                    logger.debug("Client disconnect".to_string());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Error: cannot write: {}", e),
                    ));
                }
            }
            _ => {
                logger.debug(format!("control type number: {:?}", control_type::PINGREQ));
                logger.debug("Id not match with any control packet".to_string());
                return Err(Error::new(
                    ErrorKind::Other,
                    "Error: cannot match with any control type".to_string(),
                ));
            }
        }

        Ok(client_id.to_string())
    }

    fn send_last_will(
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
        tx_server: Sender<Vec<String>>,
        client_id: String,
        logger: &Logger,
    ) {
        Server::act_on_last_will(
            hash_server_connections.clone(),
            tx_server,
            client_id.clone(),
            logger,
        );
        Server::clear_last_will(hash_server_connections, client_id);
    }

    fn clear_last_will(
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
        client_id: String,
    ) {
        // clear last will from hash_server_connections
        hash_server_connections
            .lock()
            .unwrap()
            .entry(client_id)
            .and_modify(|e| {
                let will_tuple = &mut e.1;
                will_tuple.0 = "".to_string();
                will_tuple.1 = "".to_string();
            });
    }

    fn act_on_last_will(
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
        tx_server: Sender<Vec<String>>,
        client_id: String,
        logger: &Logger,
    ) {
        let result = hash_server_connections
            .lock()
            .unwrap()
            .get(&client_id)
            .map(|e| e.1.clone());

        match result {
            Some(will_tuple) => {
                // if last will topic is not empty send last will to subscribed clients with publish
                if !will_tuple.0.is_empty() {
                    logger.debug(format!(
                        "Last will statement topic: {} from client ID: {}",
                        will_tuple.0, client_id
                    ));
                    let msg_server: Vec<String> = vec![
                        "publish".to_string(),
                        0.to_string(),
                        1.to_string(),
                        0.to_string(),
                        will_tuple.0,
                        will_tuple.1,
                    ];
                    match tx_server.send(msg_server) {
                        Ok(_) => {
                            logger.debug(format!(
                                "Message sent to server to process publish for client last will: {}",
                                client_id
                            ));
                        }
                        Err(e) => {
                            logger.debug(format!(
                                "Error sending message to server to process publish from client las will: {} error: {}",
                                client_id ,e
                            ));
                        }
                    };
                }
            }
            None => {
                logger.debug(format!("Client {} has no last will", client_id));
            }
        }
    }

    fn get_id_server_connections(
        client_id: String,
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
    ) -> Result<bool> {
        let hash = hash_server_connections.lock().unwrap();
        Ok(hash.contains_key(&client_id))
    }

    fn _get_hash_topics(topic: &str, hash_topics: Arc<Mutex<HashTopics>>) -> Result<bool> {
        let hash = hash_topics.lock().unwrap();
        Ok(hash.contains_key(topic))
    }

    fn message_handler(
        _tx_server: Arc<Mutex<Sender<Vec<String>>>>,
        rx_server: Arc<Mutex<Receiver<Vec<String>>>>,
        hash_topics: Arc<Mutex<HashTopics>>,
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
        logger: Arc<Logger>,
    ) {
        let _handle = thread::Builder::new()
            .name("Thread: Message handler".to_string())
            .spawn(move || loop {
                let rx_server_guard = rx_server.lock().unwrap();
                match rx_server_guard.recv() {
                    Ok(msg) => {
                        logger.debug(format!("Thread message handler received topic: {:?}", msg));
                        let packet_type = msg[0].as_str();
                        match packet_type {
                            // si es publish debe tomar el array de hash topic, iterarlo y cada tx de ese array debe ejercutar send con el packet valuede un publish packet
                            "publish" => {
                                // message = [ packet_type, dup, qos, retain, topic, message ]
                                let dup = msg[1].parse::<u8>().unwrap();
                                let qos = msg[2].parse::<u8>().unwrap();
                                let retain = msg[3].parse::<u8>().unwrap();
                                let topic = &msg[4];
                                let message = &msg[5];
                                // create new packet identifier
                                let mut rng = rand::thread_rng();
                                let packet_identifier: u16 = rng.gen();
                                if !topic.is_empty() {
                                    hash_topics
                                        .lock()
                                        .unwrap()
                                        .entry(topic.to_string())
                                        .and_modify(|vector| {
                                            logger.debug(format!(
                                                "Found {} subscriptors for topic: {}",
                                                vector.0.len(),
                                                topic
                                            ));
                                            if retain == 1 {
                                                logger.debug(format!(
                                                    "Saving Retain message for topic: {}",
                                                    topic
                                                ));
                                                vector.1 = message.to_string();
                                            }
                                            for val in &vector.0 {
                                                let tx = val.1.clone();
                                                let packet =
                                                    Packet::<VariableHeader, Payload>::new();
                                                let packet = packet.publish(
                                                    dup,
                                                    qos,
                                                    retain,
                                                    packet_identifier,
                                                    topic.to_string(),
                                                    message.to_string(),
                                                );
                                                tx.send(packet.value()).unwrap_or_else(|_| {
                                                    panic!(
                                                        "Cannot proccess publish message {:?}",
                                                        msg
                                                    )
                                                });
                                            }
                                        })
                                        .or_insert_with(|| {
                                            logger.debug(format!(
                                                "No subscriptors for topic: {}",
                                                topic
                                            ));
                                            if retain == 1 {
                                                logger.debug(format!(
                                                    "Saving Retain message for topic: {}",
                                                    topic
                                                ));
                                                return (vec![], message.to_string())
                                            }
                                            (vec![], "".to_string())
                                        });
                                } else {
                                    logger.debug(
                                        "Publish on server received a Topic that is is empty"
                                            .to_string(),
                                    );
                                }
                            }

                            // si es subscribe debe guardar el topic name en el hash de topic y asignar el tx obtenido mediante el peer addr sumistrado en msg con el hash de connection
                            // si ya se encuentra el topic name debe hacer push del tx
                            "subscribe" => {
                                // message = [ packet_type, client_id, packet_id, topic_name ]
                                let client_id = msg[1].as_str();
                                let _packet_id = (msg[2].as_bytes()[0] as u16) << 8
                                    | msg[2].as_bytes()[1] as u16;
                                let topic = &msg[3];

                                if !client_id.is_empty() {
                                    let value = hash_server_connections
                                        .lock()
                                        .unwrap()
                                        .get(client_id)
                                        .unwrap()
                                        .0
                                        .tx
                                        .clone();
                                    let tx = &*value.lock().unwrap();
                                    hash_topics
                                        .lock()
                                        .unwrap()
                                        .entry(topic.to_string())
                                        .and_modify(|vector| {
                                            vector.0.push((client_id.to_string(), tx.to_owned()));

                                            if !vector.1.is_empty() {
                                                logger.debug(format!(
                                                    "Sending retain message for topic: {} message: {}",
                                                    topic,
                                                    vector.1.clone()
                                                ));
                                                // send to this client the last retained message
                                                let packet =
                                                    Packet::<VariableHeader, Payload>::new();
                                                let packet = packet.publish(
                                                    0,
                                                    1,
                                                    1,
                                                    0,
                                                    topic.to_string(),
                                                    vector.1.to_string(),
                                                );
                                                tx.send(packet.value()).unwrap_or_else(|_| {
                                                    panic!(
                                                        "Cannot proccess subscribe message {:?}",
                                                        msg
                                                    )
                                                });
                                            }

                                        })
                                        .or_insert_with(|| {
                                            (vec![(client_id.to_string(), tx.to_owned())], "".to_string())
                                        });
                                } else {
                                    logger.debug("Cannot find client Identified".to_string());
                                }
                            }

                            "unsubscribe" => {
                                // unsuscribe client_id packet_identifier topic
                                let client_id = msg[1].as_str();
                                let _packet_id = (msg[2].as_bytes()[0] as u16) << 8
                                    | msg[2].as_bytes()[1] as u16;
                                let topic = &msg[3];

                                if !client_id.is_empty() {
                                    hash_topics
                                        .lock()
                                        .unwrap()
                                        .entry(topic.to_string())
                                        .and_modify(|vector| {
                                            vector.0.retain(|entry| entry.0 != client_id);
                                        });
                                }
                                logger.debug(format!(
                                    "Unsubscribed topic_name: {} for client id: {}",
                                    topic, client_id
                                ));
                            }
                            "request_clean_session" => {
                                // request_clean_session client_id 
                                let client_id = msg[1].as_str();

                                // unsubscribe client_id from all topics
                                if !client_id.is_empty() {
                                    hash_topics
                                        .lock()
                                        .unwrap()
                                        .retain(|_, vector| {
                                            vector.0.retain(|entry| entry.0 != client_id);
                                            !vector.0.is_empty()
                                        });
                                    logger.debug(format!(
                                        "Request Clean session for client id: {} success",
                                        client_id
                                    ));
                                }
                            }
                            _ => {
                                logger.debug(format!(
                                    "Not received any known packet type, received: {}",
                                    packet_type
                                ));
                            }
                        };
                        // logger.debug("Thread message handler update topic hash".to_string());
                        // for (key, value) in hash_topics.lock().unwrap().iter() {
                        //     logger.debug(format!("{:?} - {:#?}", key, value));
                        //     logger.debug(format!("{:?}", value));
                        //     logger.debug("printing following topics".to_string());
                        // }
                        // println!("ending printing topics");
                    }
                    Err(e) => {
                        logger.debug(format!("Thread message handler got an error: {:?}", e));
                        break;
                    }
                };
                drop(rx_server_guard);
                thread::sleep(Duration::from_millis(10));
            });
    }
}
