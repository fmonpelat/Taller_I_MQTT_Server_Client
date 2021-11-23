use core::time;
use std::io::{ErrorKind, Read, Result, Write,Error};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{thread};
use crate::logger::{Logger, Logging};
use mqtt_packet::mqtt_packet_service::{Packet, ServerPacket, Utils};
use mqtt_packet::mqtt_packet_service::header_packet::{control_type};
use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::{VariableHeader, connect_ack_flags, connect_return};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};//, channel};

type HashPersistanceConnections= HashMap<SocketAddr, (Sender<String>, Receiver<String>)>; //ver ip addres para u8
type HashServerConnections =  HashMap<SocketAddr, Sender<SocketAddr>>;
type HashTopics = HashMap<String, Vec<Sender<String>>>;
#[derive(Clone)]
pub struct Server {
	server_address: Arc<String>, 
  server_port: Arc<String>, 
	logger: Arc<Logger>,
  hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>, 
  hash_server_connections: Arc<Mutex<HashServerConnections>>, 
  hash_topics: Arc<Mutex<HashTopics>>,
}

#[allow(clippy::unit_arg)]
impl Server {
  pub fn new (server_address: String, server_port: String, file_source: & str) -> Server {
    let hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>> = Arc::new(Mutex::new(HashMap::new()));
    let hash_server_connections: Arc<Mutex<HashServerConnections>> = Arc::new(Mutex::new(HashMap::new()));
    let hash_topics: Arc<Mutex<HashTopics>> = Arc::new(Mutex::new(HashMap::new()));
    Server{
      server_address: Arc::new(server_address),
      server_port: Arc::new(server_port),
	    logger: Arc::new(Logger::new(file_source)),
      hash_persistance_connections,
      hash_server_connections,
      hash_topics
    }
  }

  fn handle_client(peer: String, mut stream: TcpStream, logger: Arc<Logger>, hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>) -> Result<JoinHandle<()>> {
    
    fn _handle_client_(mut stream: TcpStream, logger: Arc<Logger>, hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>) -> Result<()> {
      let mut buff = [0_u8; 1024];

      Ok( loop {
        match stream.read(&mut buff) {
          Ok(_size) => {
            if _size > 0 {
              let packet_identifier = buff[0];
              logger.debug(format!("Check if a MQTT PACKET is received: {:?}",packet_identifier));
              if Packet::<VariableHeader, Payload>::is_mqtt_packet(&buff) {
                logger.debug(format!("Found a MQTT packet: {:?}",packet_identifier));
                match Server::handle_packet(buff.to_vec(), & mut stream, &logger, hash_persistance_connections.clone()) {
                  Ok(_) => {
                    logger.debug(format!("Peer {} succefully connect with server",stream.peer_addr()?));
                    let mut copy_buff = [0_u8; 1024];
                    buff = Server::recalculate_buff( &mut buff, &mut copy_buff);
                    continue;
                  },
                  Err(e) => {
                    logger.debug(format!("Error: {}", e));
                  },
                }
              } else {
                    logger.debug("Clean buffer to continue reading from stream".to_string());
                    //drop(buff);
                    buff = [0_u8; 1024];
              };
              
              thread::sleep(time::Duration::from_millis(2000));            
            };
          },
          Err(_) => {
            println!(
              "An error occurred, terminating connection with {}",
              stream.peer_addr()?
            );
            stream.shutdown(Shutdown::Both)?;
            break
          },
        }
        })
    }

    let handle = thread::Builder::new().name("thread peer: ".to_string()+peer.to_string().as_str())
          .spawn( move || {
            // connection succeeded
            println!("Connection from {}", peer);
            //let mut locked = clone_mutex.lock().unwrap();
            match _handle_client_(stream, logger, hash_persistance_connections) {
              Ok(_) => {
                  println!("Connection with {} closed", peer);
              },
              Err(e) => {
                  println!("Error: {}", e);
              },
            }
          });
    handle
  }


  pub fn listening(&self)-> Result<()> {

    let server_address = Arc::clone(&self.server_address);
    let server_port = Arc::clone(&self.server_port);
	  self.logger.debug("ready to binding".to_string());
    self.logger.info(format!("server address: {:?}", server_address.to_string() + ":" + &server_port.as_str()));
    let address = format!("{}:{}", server_address, server_port);
    let listener = TcpListener::bind( address)?;
    // accept connections and process them, spawning a new thread for each one
    self.logger.debug("start binding".to_string());
    println!("Server listening on port {}", self.server_port);

    self.logger.info("starting listening to clients".to_string());
    let server_mutex = Arc::new(Mutex::new(self)); // moved self to a Arc Mutex to access the server struct
    for stream in listener.incoming() {
      let _clone_server = Arc::clone(&server_mutex);
      match stream {
        Ok(stream) => {
          let peer = stream.peer_addr()?;
          let this = server_mutex.lock().unwrap();
          let logger = this.logger.clone();
          let _persistance_connections = Arc::clone(&this.hash_persistance_connections);
          logger.info(format!("New client connected: {}", peer));
          let _handle = Server::handle_client(peer.to_string(), stream, logger, _persistance_connections);
          
        }
        Err(e) => { /* connection failed */
          println!("Error: {}", e);
          break
        }
      }
    }
    // close the socket server
    drop(listener);
    server_mutex.lock().unwrap().logger.info("Server terminated.".to_string()); //ver porque no se escribe esta linea no se escribe en el log
    Ok(())

  }

  fn handle_packet(buff: Vec<u8>, stream: &mut TcpStream, logger: &Arc<Logger>, hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>) -> Result<&'static str>{

    let packet_id = buff[0] & 0xF0;
    if (packet_id == control_type::CONNECT) && Server::get_id_persistance_connections(stream, hash_persistance_connections)?{
      logger.debug("Client already connected".to_string());
      return Err(Error::new(ErrorKind::Other, "Error, client already connected"))
    }
    // check other packets type
    match packet_id {
      control_type::CONNECT  => {
        logger.info("Connect packet received".to_string());
        logger.debug(format!("Peer mqtt connected: {}, with action type: {} ",stream.peer_addr()?,packet_id));
        
        let unvalued_packet = Packet::<VariableHeader, Payload>::unvalue(buff);        
        let _client_identifier: String = unvalued_packet.payload.client_identifier;

        //TODO ADD PEER ID AND TX TO HASH
        //hash_server_connections.insert( &stream.peer_addr()?, tx );
        let packet = Packet::<VariableHeader, Payload>::new();
        let packet = packet.connack(connect_ack_flags::SESSION_PRESENT, connect_return::ACCEPTED);
        if let Err(e) = stream.write_all(&packet.value()) {
          logger.debug("Client disconnect".to_string());
          return Err(Error::new(ErrorKind::Other, format!("Error: cannot write: {}",e)))
        }
      },
      control_type::PUBLISH  => {
          logger.debug("Publish packet received".to_string());
          logger.debug(format!("Peer mqtt publish: {:?}",stream.peer_addr()?));
      },
      _ => {
        logger.debug("Id not match with any control packet".to_string());
        return Err(Error::new(ErrorKind::Other, format!("Error: cannot match with any control type")))
      }
    }
  
    Ok("Successfully handle packet")
  }

  fn recalculate_buff(buff: &mut [u8; 1024], copy_buff: &mut [u8; 1024]) -> [u8; 1024] {
    let mut buff_readed:usize = 0;
    let remaining_len = Packet::<VariableHeader, Payload>::get_packet_length(&buff[1..buff.len()].to_vec(),&mut buff_readed);               
    copy_buff[(remaining_len+buff_readed+1)..buff.len()].clone_from_slice(&buff[(remaining_len+buff_readed+1)..buff.len()]);
    *copy_buff
  }
 

  fn get_id_persistance_connections(stream: &mut TcpStream, hash_persistance_connections: Arc<Mutex<HashPersistanceConnections>>) -> Result<bool>{
    let hash =  hash_persistance_connections.lock().unwrap();
    println!("contains key result: {:?}", hash.contains_key(&stream.peer_addr()?));
    Ok(hash.contains_key(&stream.peer_addr()?))
  }

  fn get_hash_server_connections( &self ,stream: &mut TcpStream ) -> Result<bool>{
    let hash =  self.hash_server_connections.lock().unwrap();
    println!("contains key result: {:?}", hash.contains_key(&stream.peer_addr()?));
    Ok(hash.contains_key(&stream.peer_addr()?))
  }

  fn get_hashTopics( &self , topic: &String ) -> Result<bool>{
    let hash =  self.hash_topics.lock().unwrap();
    println!("contains key result: {:?}", hash.contains_key(topic));
    Ok(hash.contains_key(topic))
  }

}