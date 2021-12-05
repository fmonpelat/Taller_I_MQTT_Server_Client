use crate::logger::{Logger, Logging};
use mqtt_packet::mqtt_packet_service::header_packet::control_flags::{self};
use mqtt_packet::mqtt_packet_service::header_packet::{control_type, PacketHeader};
use mqtt_packet::mqtt_packet_service::payload_packet::{Payload, PublishPayload, SuscribePayload};
use mqtt_packet::mqtt_packet_service::variable_header_packet::{
    connect_ack_flags, connect_return, VariableHeader, VariableHeaderPacketIdentifier,
    VariableHeaderPublish,
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

type HashPersistanceConnections = HashMap<String, JoinHandle<()>>; // la clave es el ip address
type HashServerConnections = HashMap<String, HandleClientConnections>; // la clave es el client_id de mqtt
type HashTopics = HashMap<String, Vec<Sender<Vec<u8>>>>;
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
}
#[derive(Clone, Debug)]
pub struct HandleClientConnections {
    tx: Arc<Mutex<Sender<Vec<u8>>>>,
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    peer: Arc<Mutex<String>>,
}

#[allow(clippy::unit_arg)]
impl Server {
    pub fn new(server_address: String, server_port: String, file_source: &str) -> Server {
        let hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>> =
            Arc::new(Mutex::new(HashMap::new()));
        let hash_server_connections: Arc<Mutex<HashServerConnections>> =
            Arc::new(Mutex::new(HashMap::new()));
        let hash_topics: Arc<Mutex<HashTopics>> = Arc::new(Mutex::new(HashMap::new()));
        let (tx_server, rx_server) = channel::<Vec<String>>();
        //TODO Check return and see how call to the function
        let server = Server {
            server_address: Arc::new(server_address),
            server_port: Arc::new(server_port),
            logger: Arc::new(Logger::new(file_source, true)),
            hash_persistance_connections,
            hash_server_connections,
            hash_topics,
            tx_server: Arc::new(Mutex::new(tx_server)),
            rx_server: Arc::new(Mutex::new(rx_server)),
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
        tx_server: Sender<Vec<String>>,
    ) -> Result<JoinHandle<()>> {
        fn _handle_client_(
            mut stream: TcpStream,
            logger: Arc<Logger>,
            hash_server_connections: Arc<Mutex<HashServerConnections>>,
            hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>,
            client_connections: HandleClientConnections,
            tx_server: Sender<Vec<String>>,
        ) -> Result<()> {
            let mut buff = [0_u8; 1024];
            let mut _client_id = String::new();
            Ok(loop {
                match stream.read(&mut buff) {
                    Ok(_size) => {
                        if _size > 0 {
                            let control_type = buff[0];
                            logger.debug(format!(
                                "Check if a MQTT PACKET is received: {:?}",
                                control_type
                            ));
                            if Packet::<VariableHeader, Payload>::is_mqtt_packet(&buff) {
                                logger.debug(format!("Found a MQTT packet: {:?}", control_type));
                                match Server::handle_packet(
                                    buff.to_vec(),
                                    &mut stream,
                                    logger.clone(),
                                    hash_server_connections.clone(),
                                    hash_persistance_connections.clone(),
                                    &client_connections,
                                    tx_server.clone(),
                                    &mut _client_id,
                                ) {
                                    Ok(client_id) => {
                                        logger.debug(format!(
                                            "Packet from peer {} and client id: {} has been processed",
                                            stream.peer_addr()?, client_id
                                        ));
                                        logger.debug("Cleaning buffer".to_string());
                                        buff = [0_u8; 1024];
                                    }
                                    Err(e) => {
                                        logger.debug(format!("Error: {}", e));
                                    }
                                }
                            } else {
                                logger.debug(
                                    "Clean buffer to continue reading from stream".to_string(),
                                );
                                buff = [0_u8; 1024];
                            };
                        };
                    }
                    Err(_) => {
                        logger.debug(format!(
                            "An error occurred, terminating connection with {}",
                            stream.peer_addr()?
                        ));
                        stream.shutdown(Shutdown::Both)?;
                        break;
                    }
                }

                // TODO, read rx channel of this thread and act like a proxy (write drectly to stream)
            })
        }

        let (tx, rx) = channel::<Vec<u8>>();

        let handle_client_connections = HandleClientConnections {
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            peer: Arc::new(Mutex::new(peer.to_string())),
        };

        let handle = thread::Builder::new()
            .name("thread peer: ".to_string() + peer.to_string().as_str())
            .spawn(move || {
                // connection succeeded
                logger.debug(format!("Connection from {}", peer));
                match _handle_client_(
                    stream,
                    logger.clone(),
                    hash_server_connections,
                    hash_persistance_connections,
                    handle_client_connections,
                    tx_server,
                ) {
                    Ok(_) => {
                        logger.debug(format!("Connection with {} closed", peer));
                    }
                    Err(e) => {
                        logger.debug(format!("Error: {}", e));
                    }
                }
                //TODO listen to rx handle client connection to send
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
                        logger,
                        this.hash_server_connections.clone(),
                        this.hash_persistance_connections.clone(),
                        tx.clone(),
                    );
                    self.hash_persistance_connections
                        .lock()
                        .unwrap()
                        .insert(peer.ip().to_string(), _handle.unwrap());
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

    fn handle_packet(
        buff: Vec<u8>,
        stream: &mut TcpStream,
        logger: Arc<Logger>,
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
        hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>,
        client_connections: &HandleClientConnections,
        tx_server: Sender<Vec<String>>,
        client_id: &mut String,
    ) -> Result<String> {
        let packet_id = buff[0] & 0xF0;
        let peer_addr = stream.peer_addr()?;

        if packet_id == control_type::CONNECT as u8 {
            let unvalued_packet = Packet::<VariableHeader, Payload>::unvalue(buff.clone());
            let client_identifier: String = unvalued_packet.payload.client_identifier;
            logger.debug(format!(
                "Client Connection with Client identifier: {}, verifying that was connected...",
                client_identifier
            ));

            if Server::get_id_persistance_connections(
                client_identifier,
                hash_server_connections.clone(),
            )
            .unwrap()
            {
                logger.debug(format!("Client already connected clientId: {}", client_id));
                return Err(Error::new(
                    ErrorKind::Other,
                    "Error, client already connected",
                ));
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
                let client_id = client_identifier;

                // TODO: check retain if we need to persist the client_id
                hash_server_connections
                    .lock()
                    .unwrap()
                    .insert(client_id, client_connections.clone());

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
                if unvalue.header.control_flags == control_flags::QOS1 {
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
                let msg_server: Vec<String> = vec![
                    "publish".to_string(),
                    unvalue.header.get_qos().to_string(),
                    if unvalue.header.get_dup() { 1 } else { 0 }.to_string(),
                    if unvalue.header.get_retain() { 1 } else { 0 }.to_string(),
                    String::from_utf8_lossy(&unvalue.variable_header.topic_name).to_string(),
                    unvalue.payload.message,
                ];
                tx_server
                    .send(msg_server.clone())
                    .unwrap_or_else(|_| panic!("Cannot proccess publish message {:?}", msg_server));
            }
            control_type::DISCONNECT => {
                // TODO investigate what kind of action need to take
                logger.debug("Disconnect packet received".to_string());
                logger.info(format!("Peer {:?} will be disconnected ", peer_addr));

                hash_persistance_connections
                    .lock()
                    .unwrap()
                    .remove(&peer_addr.ip().to_string());
                // TODO Check if need to clean hash persistance connection as well
                logger.debug(format!(
                    "Peer {} has been removed from hash server connections",
                    peer_addr
                ));
            }
            control_type::SUBSCRIBE => {
                logger.debug("Suscribe packet received".to_string());
                logger.debug(format!("Peer mqtt suscribe: {:?}", peer_addr));

                let unvalue =
                    Packet::<VariableHeaderPacketIdentifier, SuscribePayload>::unvalue(buff);
                let packet_identifier = unvalue.variable_header.packet_identifier;
                let qos = unvalue.payload.qos;
                let topics = unvalue.payload.topic_filter;

                // enviando al tx del server los topics suscriptos
                logger.debug("Sending tx server to the subcribed topics".to_string());
                for topic in topics {
                    let msg_server = vec![
                        "subscribe".to_string(),
                        client_id.to_string(),
                        packet_identifier.to_string(),
                        topic.to_string(),
                    ];
                    tx_server.send(msg_server.clone()).unwrap_or_else(|_| {
                        panic!("Cannot proccess subscribe message {:?}", msg_server)
                    });
                }
                // enviar el suback
                logger.debug("Sending suback packet to client".to_string());
                let packet = Packet::<VariableHeader, Payload>::new();
                let packet = packet.suback(packet_identifier, qos);
                if let Err(e) = stream.write_all(&packet.value()) {
                    logger.debug("Client disconnect".to_string());
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("Error: cannot write: {}", e),
                    ));
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

    fn get_id_persistance_connections(
        ip_addr: String,
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
    ) -> Result<bool> {
        let hash = hash_server_connections.lock().unwrap();
        Ok(hash.contains_key(&ip_addr))
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
                                let dup = msg[1].as_bytes()[0];
                                let qos = msg[2].as_bytes()[0];
                                let retain = msg[3].as_bytes()[0];
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
                                            for tx in vector {
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
                                let client_id = msg[2].as_str();
                                let _packet_id = msg[3].as_bytes()[0] as u16;
                                let topic = &msg[4];

                                if !client_id.is_empty() {
                                    hash_topics
                                        .lock()
                                        .unwrap()
                                        .entry(topic.to_string())
                                        .and_modify(|vector| {
                                            let value = hash_server_connections
                                                .lock()
                                                .unwrap()
                                                .get(client_id)
                                                .unwrap()
                                                .tx
                                                .clone();
                                            let tx = &*value.lock().unwrap();
                                            vector.push(tx.to_owned());
                                        });
                                } else {
                                    logger.debug("Cannot find client Identified".to_string());
                                }
                            }
                            _ => {
                                logger.debug("Not received any packet type".to_string());
                            }
                        };
                        logger.debug("Thread message handler update topic hash".to_string());
                        for (key, value) in hash_topics.lock().unwrap().iter() {
                            logger.debug(format!("{:?} - {:#?}", key, value));
                            logger.debug(format!("{:?}", value));
                            logger.debug("printing following topics".to_string());
                        }
                    }
                    Err(e) => {
                        logger.debug(format!("Thread message handler got an error: {:?}", e));
                        break;
                    }
                };
                thread::sleep(Duration::from_millis(50));
            });
    }
}
