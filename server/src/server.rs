use core::time;
use std::io::{Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc};
use std::{thread};
use crate::logger::{Logger, Logging};
use mqtt_packet::mqtt_packet_service::{Packet, Utils};
use mqtt_packet::mqtt_packet_service::header_packet::{control_type};
use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::{VariableHeader};

pub struct Server {
	server_address: String, 
  server_port: String, 
	logger: Arc<Logger>,   
  }

#[allow(clippy::unit_arg)]
impl Server {
  pub fn new(server_address: String, server_port: String, file_source: & str) -> Self {
    Server{
      server_address,
      server_port,
	    logger: Arc::new(Logger::new(file_source)),
      //TODO HASH PEER/ TX
      //TODO HASH CLIENT/CHANNEL TUPLE
      //TODO HASH TOPICS
    }
  }

  fn handle_client( mut stream: TcpStream, logger: Arc<Logger>) -> Result<()> {
    let mut buff = [0_u8; 7]; // using 50 u8 buffer
    
    Ok( loop {
      match stream.read(&mut buff) {
        Ok(_size) => {
          println!("{:?}", buff);
          if _size > 0 {
            let packet_identifier = buff[0];
            logger.debug(format!("Check if a MQTT PACKET is received: {:?}",packet_identifier));
            if Packet::<VariableHeader, Payload>::is_mqtt_packet(&buff) {
              logger.debug(format!("Found a MQTT packet: {:?}",packet_identifier));
              match Server::handle_packet(packet_identifier, & mut stream, &logger) {
                Ok(_) => {
                  logger.debug(format!("Connection with {} closed", packet_identifier));
                  // TODO let remaining_len = Packet::<VariableHeader, Payload>::get_packet_length(&buff[1..buff.len()].to_vec());
                  // TODO buff=buff[(remaining_len+1)..buff.len()]
                  // todo check buff
                },
                Err(_) => {
                  logger.debug("Error process stream".to_string());
                },
            }
            //else borrar buf, loopear de nuevo buff=vector vacio
            }
            thread::sleep(time::Duration::from_millis(2000));            
          }
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
    //let found_id = Server::packet_identifier(packet_id);
    //TODO SEE IF CLIENT/PEER IS CONNECTED --> CHECK HASH
    // CHECK CONTROL TYPE IS A CONNECT BEFORE MATCH
    match packet_id {
      control_type::CONNECT => { 
        //TODO CHECK VALID CLIENT
        println!("Connect packet received \n");
        logger.debug(format!("Peer mqtt connected: {}",stream.peer_addr()?));
        //TODO ADD PEER ID AND TX TO HASH , 
        //let packet = Packet::<VariableHeader, Payload>::new();
        //let packet =
        //    packet.connack(connect_ack_flags::SESSION_PRESENT, connect_return::ACCEPTED);
        //PACKET.VALUE() , TO_BYTE IS REQUIRE
        if let Err(e) = stream.write_all(b"32") { 
          logger.debug("Client disconnect".to_string());
          return Err(e) // Send client id when write_all fails
        }
      },
      control_type::PUBLISH  => {
          logger.debug("Publish packet received".to_string());
          logger.debug(format!("Peer mqtt publish: {:?}",stream.peer_addr()?));
      },
      _ => {
        logger.debug("Id not match with any control packet".to_string());
      }
    }
    Ok(())
  }
}