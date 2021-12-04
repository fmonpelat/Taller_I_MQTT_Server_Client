use crate::logger::{Logger, Logging};
use mqtt_packet::mqtt_packet_service::header_packet::control_flags::{self};
use mqtt_packet::mqtt_packet_service::header_packet::control_type;
use mqtt_packet::mqtt_packet_service::payload_packet::{
    suback_return_codes, Payload, PublishPayload, SuscribePayload, SubackPayload,
};
use mqtt_packet::mqtt_packet_service::variable_header_packet::{
    connect_ack_flags, connect_return, VariableHeader, VariableHeaderPacketIdentifier,
    VariableHeaderPublish,
};
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet, ServerPacket, Utils};
use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Result, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::thread::{self};
use std::time::Duration;

type HashPersistanceConnections = HashMap<String, JoinHandle<()>>;  // la clave es el client_id de mqtt
type HashServerConnections = HashMap<String, HandleClientConnections>; // la clave es el client_id de mqtt
type HashTopics = HashMap<String, Vec<Sender<Vec<String>>>>;
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
    tx: Arc<Mutex<Sender<String>>>,
    rx: Arc<Mutex<Receiver<String>>>,
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
        );
        server
    }

    fn handle_client(
        peer: String,
        stream: TcpStream,
        logger: Arc<Logger>,
        hash_server_connections: Arc<Mutex<HashServerConnections>>,
        tx_server: Sender<Vec<String>>,
    ) -> Result<JoinHandle<()>> {
        fn _handle_client_(
            mut stream: TcpStream,
            logger: Arc<Logger>,
            hash_server_connections: Arc<Mutex<HashServerConnections>>,
            server_connections: HandleClientConnections,
            tx_server: Sender<Vec<String>>,
            tx_client: Sender<String>,
        ) -> Result<()> {
            let mut buff = [0_u8; 1024];
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
                                    &logger,
                                    &hash_server_connections,
                                    &server_connections,
                                    tx_server.clone(),
                                ) {
                                    Ok(_) => {
                                        logger.debug(format!(
                                            "Packet from peer {} has been processed",
                                            stream.peer_addr()?
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

                            // thread::sleep(time::Duration::from_millis(2000));
                        };
                    }
                    Err(_) => {
                        println!(
                            "An error occurred, terminating connection with {}",
                            stream.peer_addr()?
                        );
                        stream.shutdown(Shutdown::Both)?;
                        break;
                    }
                }

                // TODO, read rx channel and write to stream
            })
        }

        let (tx, rx) = channel::<String>();

        let handle_client_connections = HandleClientConnections {
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            peer: Arc::new(Mutex::new(peer.to_string())),
        };

        let handle = thread::Builder::new()
            .name("thread peer: ".to_string() + peer.to_string().as_str())
            .spawn(move || {
                // connection succeeded
                println!("Connection from {}", peer);
                match _handle_client_(
                    stream,
                    logger,
                    hash_server_connections,
                    handle_client_connections,
                    tx_server,
                    handle_client_connections.tx,

                ) {
                    Ok(_) => {
                        println!("Connection with {} closed", peer);
                    }
                    Err(e) => {
                        println!("Error: {}", e);
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
        println!("Server listening on port {}", self.server_port);

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
                        tx.clone(),
                    );
                    self.hash_persistance_connections
                        .lock()
                        .unwrap()
                        .insert(peer.to_string(), _handle.unwrap());
                }
                Err(e) => {
                    /* connection failed */
                    println!("Error: {}", e);
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
        logger: &Arc<Logger>,
        hash_server_connections: &Arc<Mutex<HashServerConnections>>,
        server_connections: &HandleClientConnections,
        tx_server: Sender<Vec<String>>,
        hash_topics: Arc<Mutex<HashTopics>>,
    ) -> Result<&'static str> {
        let packet_id = buff[0] & 0xF0;
        let peer_addr = stream.peer_addr()?;
        let connect_packet = Packet::<VariableHeader,Payload>::unvalue(buff);

        if (packet_id == control_type::CONNECT)
            && Server::get_id_server_connections(peer_addr, hash_server_connections)?
        {
            logger.debug("Client already connected".to_string());
            return Err(Error::new(
                ErrorKind::Other,
                "Error, client already connected",
            ));
            // TODO IF NEED TO REMOVE PEER FROM HASH SERVER CONNECTION
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
                let _client_identifier: String = unvalued_packet.payload.client_identifier;

                hash_server_connections
                    .lock()
                    .unwrap()
                    .insert(peer_addr.to_string(), server_connections.clone());

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
                    // TO DO investigate what kind of action need to take
                    logger.debug("Identified QoS0 flag. Nothing to do".to_string());
                }
                let msg_server: Vec<String> = vec![
                    "publish".to_string(),
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
                hash_server_connections.lock().unwrap().remove(&peer_addr);
                // TODO Check if need to clean hash persistance connection as well
                logger.debug(format!(
                    "Peer {} has been removed from hash server connections",
                    peer_addr
                ));
            }
            control_type::SUBSCRIBE => {
                logger.debug("Suscribe packet received".to_string());
                logger.debug(format!("Peer mqtt suscribe: {:?}", peer_addr));

                let unvalue = Packet::<VariableHeaderPacketIdentifier, SuscribePayload>::unvalue(buff);
                let packet_identifier = unvalue.variable_header.packet_identifier;
                let qos = unvalue.payload.qos;
                
                let topics = unvalue.payload.topic_filter;
                for topic in &topics{
                    hash_topics
                                .lock()
                                .unwrap()
                                .entry(topic.to_string())
                                .and_modify(|vector| {
                                    vector.push(tx_server.clone())
                                });
                }
                self.hash_persistance_connections
                                .lock()
                                .unwrap()
                                .insert(peer_addr, tx_server);
                

                let packet = Packet::<VariableHeader, Payload>::new();
                let packet = packet.suback(packet_identifier, qos);
                // TODO add topic and tx channel into hash topics
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
                println!("control type number: {:?}", control_type::PINGREQ);
                logger.debug("Id not match with any control packet".to_string());
                return Err(Error::new(
                    ErrorKind::Other,
                    "Error: cannot match with any control type".to_string(),
                ));
            }
        }

        Ok("Successfully handle packet")
    }

    //fn get_id_persistance_connections(
    //    peer_addr: SocketAddr,
    //    hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>,
    //) -> Result<bool> {
    //    let hash = hash_persistance_connections.lock().unwrap();
    //    Ok(hash.contains_key(&peer_addr))
    //}

    fn get_id_server_connections(
        peer_addr: SocketAddr,
        hash_server_connections: &Arc<Mutex<HashServerConnections>>,
    ) -> Result<bool> {
        let hash = hash_server_connections.lock().unwrap();
        Ok(hash.contains_key(&peer_addr))
    }

    fn _get_hash_topics(topic: &str, hash_topics: Arc<Mutex<HashTopics>>) -> Result<bool> {
        let hash = hash_topics.lock().unwrap();
        Ok(hash.contains_key(topic))
    }
    fn message_handler(
        tx_server: Arc<Mutex<Sender<Vec<String>>>>,
        rx_server: Arc<Mutex<Receiver<Vec<String>>>>,
        hash_topics: Arc<Mutex<HashTopics>>,
    ) {
        let _handle = thread::Builder::new()
            .name("Thread: Message handler".to_string())
            .spawn(move || loop {
                let rx_server_guard = rx_server.lock().unwrap();
                match rx_server_guard.recv() {
                    Ok(msg) => {
                        println!("Thread message handler received topic: {:?}", msg);
                        // check if topic name exist
                        let topic_name = &msg[1];
                        //TODO verificar si el topic name es suscribe o publish
                        // si es publish debe tomar el array de hash topic, iterarlo y cada tx de ese array debe ejercutar send con el packet valuede un publish packet
                        // si es subscribe debe guardar el topic name en el hash de topic y asignar el tx obtenido mediante el peer addr sumistrado en msg con el hash de connection
                        // si ya se encuentra el topic name debe hacer push del tx
                        //if !hash_topics.lock().unwrap().contains_key(topic_name) {
                        //    let mut vec: Vec<Sender<Vec<String>>> = Vec::new();
                        //    let tx = tx_server.lock().unwrap().clone();
                        //    vec.push(tx);
                        //    hash_topics
                        //        .lock()
                        //        .unwrap()
                        //        .insert(topic_name.to_string(), vec);
                        //} else {
                        //    hash_topics
                        //        .lock()
                        //        .unwrap()
                        //        .entry(topic_name.to_string())
                        //        .and_modify(|vector| {
                        //            vector.push(tx_server.lock().unwrap().clone())
                        //        });
                        //}
                        println!("Thread message handler update topic hash ");
                        for (key, value) in hash_topics.lock().unwrap().iter() {
                            println!("{:?} - {:#?}", key, value);
                            println!("{:?}", value);
                            println!("printing following topics");
                        }
                    }
                    Err(e) => {
                        println!("Thread message handler got an error: {:?}", e);
                        break;
                    }
                };
                thread::sleep(Duration::from_millis(50));
            });
    }
}
