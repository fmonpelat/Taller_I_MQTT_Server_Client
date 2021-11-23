use core::time;
use std::io::{Read, Result, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc};
use std::{thread};
use crate::logger::{Logger, Logging};
use mqtt_packet::mqtt_packet_service::{Packet, ServerPacket, Utils};
use mqtt_packet::mqtt_packet_service::header_packet::{control_type};
use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::{VariableHeader, connect_ack_flags, connect_return};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};

type HashPersistanceConnections= HashMap<SocketAddr, (Sender<String>, Receiver<String>)>; //ver ip addres para u8
type HashServerConnections =  HashMap<SocketAddr, Sender<SocketAddr>>;
type HashTopics = HashMap<String, Vec<Sender<String>>>;
pub struct Server {
	server_address: String, 
  server_port: String, 
	logger: Arc<Logger>,
  hash_persistance_connections:HashPersistanceConnections, 
  hash_server_connections:HashServerConnections, 
  hash_topics:HashTopics,
  }

#[allow(clippy::unit_arg)]
impl Server {
  pub fn new(server_address: String, server_port: String, file_source: & str) -> Self {
    let mut hash_persistance_connections = HashPersistanceConnections::new();
    let mut hash_server_connections = HashServerConnections::new();
    let mut hash_topics = HashTopics::new();
    Server{
      server_address,
      server_port,
	    logger: Arc::new(Logger::new(file_source)),
      hash_persistance_connections,
      hash_server_connections,
      hash_topics
    }
  }

  fn handle_client( mut stream: TcpStream, logger: Arc<Logger>) -> Result<()> {
    let mut buff = [0_u8; 1024];

    Ok( loop {
      match stream.read(&mut buff) {
        Ok(_size) => {
          if _size > 0 {
            let packet_identifier = buff[0];
            logger.debug(format!("Check if a MQTT PACKET is received: {:?}",packet_identifier));
            if Packet::<VariableHeader, Payload>::is_mqtt_packet(&buff) {
              logger.debug(format!("Found a MQTT packet: {:?}",packet_identifier));
              match Server::handle_packet(packet_identifier, & mut stream, &logger) {
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


  pub fn connect(&self) -> Result<()> {
	  self.logger.debug("ready to binding".to_string());
    self.logger.info(format!("server address: {:?}",self.server_address.to_owned() + ":" + &self.server_port));
    let listener = TcpListener::bind(self.server_address.to_owned() + ":" + &self.server_port)?;
    // accept connections and process them, spawning a new thread for each one
    self.logger.debug("start binding".to_string());
    println!("Server listening on port {}", self.server_port);
    for stream in listener.incoming() {
        self.logger.info("start listening to clients".to_string());
        match stream {
            Ok(stream) => {
                let peer = stream.peer_addr()?;
                self.logger.info(format!("New connection: {}", peer));
                let logger = self.logger.clone();

                thread::Builder::new().name("thread peer: ".to_string()+peer.to_string().as_str())
                .spawn(move || {
                    // connection succeeded
                    println!("Connection from {}", peer);
                    match Server::handle_client(stream, logger) {
                        Ok(_) => {
                            println!("Connection with {} closed", peer);
                        },
                        Err(e) => {
                            println!("Error: {}", e);
                        },
                    }
                })?;
            }
            Err(e) => { /* connection failed */
                println!("Error: {}", e);
                break
            }
        }
    }
    // close the socket server
    drop(listener);
    self.logger.info("Server terminated.".to_string()); //ver porque no se escribe esta linea no se escribe en el log
    Ok(())

  }


  fn handle_packet( packet_id: u8, stream: &mut TcpStream, logger: &Arc<Logger>) -> Result<()>{
    // TODO ACCESSING TO A FIELD FAIL 
    if (packet_id == control_type::CONNECT) & Server::get_hash_server_connections(_).contains_key(&stream.peer_addr()?){
      logger.debug("Client already connected".to_string());
      return Err(()) // TODO RETURN A ESPECIFIC ERROR TYPE, MAYBE CREATE ERROR ENUM
    }
    // check other packets type
    match packet_id {
      control_type::CONNECT  => {
        logger.info("Connect packet received".to_string());
        logger.debug(format!("Peer mqtt connected: {}, with action type: {} ",stream.peer_addr()?,packet_id));
        //TODO ADD PEER ID AND TX TO HASH
        //hash_server_connections.insert( &stream.peer_addr()?, tx );
        let packet = Packet::<VariableHeader, Payload>::new();
        let packet = packet.connack(connect_ack_flags::SESSION_PRESENT, connect_return::ACCEPTED);
        if let Err(e) = stream.write_all(&packet.value()) {
          logger.debug("Client disconnect".to_string());
          return Err(e) // TODO RETURN A ESPECIFIC ERROR TYPE, MAYBE CREATE ERROR ENUM
        }
      },
      control_type::PUBLISH  => {
          logger.debug("Publish packet received".to_string());
          logger.debug(format!("Peer mqtt publish: {:?}",stream.peer_addr()?));
      },
      _ => {
        logger.debug("Id not match with any control packet".to_string());
        return Err(e) // TODO RETURN A ESPECIFIC ERROR TYPE, MAYBE CREATE ERROR ENUM
      }
    }
  
    Ok(())
  }

  fn recalculate_buff(buff: &mut [u8; 1024], copy_buff: &mut [u8; 1024]) -> [u8; 1024] {
    let mut buff_readed:usize = 0;
    let remaining_len = Packet::<VariableHeader, Payload>::get_packet_length(&buff[1..buff.len()].to_vec(),&mut buff_readed);               
    copy_buff[(remaining_len+buff_readed+1)..buff.len()].clone_from_slice(&buff[(remaining_len+buff_readed+1)..buff.len()]);
    *copy_buff
  }
 

  fn get_hash_persistance_connections( self:Server ) -> HashPersistanceConnections{
    self.hash_persistance_connections
  }

  fn get_hash_server_connections( self:Server ) -> HashServerConnections{
    self.hash_server_connections
  }

  fn get_hashTopics( self:Server ) -> HashTopics{
    self.hash_topics
  }

}