use core::time;
use std::io::{Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc};
use std::{thread};
use crate::logger::{Logger, Logging};
use mqtt_packet::mqtt_packet_service::header_packet::{control_type};

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
    }
  }

  fn handle_client( mut stream: TcpStream, logger: Arc<Logger>) -> Result<()> {
    let mut buff = [0_u8; 7]; // using 50 u8 buffer
    
    Ok( loop {
      match stream.read(&mut buff) {
        Ok(_size) => {
          println!("{:?}", buff);
          if _size > 0 {
            let packet_identifier = buff[..1].to_vec().pop(); //FM: porque no directamente haces buff[0]? eso te daria ya el u8 y no option...
            if Server::is_mqtt_packet( packet_identifier ) {
              logger.debug(format!("Found a MQTT packet: {}",packet_identifier.unwrap()));
              match Server::handle_packet(packet_identifier, & mut stream, &logger) {
                Ok(_) => {
                  logger.debug(format!("Connection with {} closed", packet_identifier.unwrap()));
                },
                Err(_) => {
                  logger.debug("Error process stream".to_string());
                },
            }
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

  fn is_mqtt_packet( packet_id:Option<u8> ) -> bool{
    let mut is_founded = false;  // 
    let id = packet_id.unwrap();
    //let found_id = Server::packet_identifier(id);
    if Server::packet_identifier(id) == id { // FM: que pasa si llega un 255 como packet identifier? lo toma como valido....
      is_founded = true 
    }
    is_founded 

    // FM: podrias haber buscado en un vector de packet identifiers validos capaz era mas facil que hacer match porque no son tantos
    // algo como:
    // let packet_identifiers = [
    //   control_type::CONNECT,
    //   control_type::CONNACK,
    //   control_type::PUBLISH,
    //   //...
    // ];
    // let found_id = packet_identifiers.iter().find(|&x| x == &id);
    // found_id.is_some()
  }

  fn packet_identifier( packet_identifier:u8) -> u8 {
    
    match packet_identifier {
      control_type::CONNECT =>{
        control_type::CONNECT   
      },
      control_type::CONNACK =>{
        control_type::CONNACK  
      },
      control_type::PUBLISH =>{
        control_type::PUBLISH    
      },
      control_type::PUBACK =>{
        control_type::PUBACK   
      },
      control_type::PUBREC =>{
        control_type::PUBREC   
      },
      control_type::PUBREL =>{
        control_type::PUBREL    
      },
      control_type::PUBCOMP =>{
        control_type::PUBCOMP    
      },
      control_type::SUBSCRIBE =>{
        control_type::SUBSCRIBE    
      },
      control_type::SUBACK =>{
        control_type::SUBACK    
      },
      control_type::UNSUBSCRIBE =>{
        control_type::UNSUBSCRIBE  
      },
      control_type::UNSUBACK =>{
        control_type::UNSUBACK   
      },
      control_type::PINGREQ =>{
        control_type::PINGREQ   
      },
      control_type::PINGRESP =>{
        control_type::PINGRESP   
      },
      control_type::DISCONNECT =>{
        control_type::DISCONNECT   
      },
      _ => {
        255 //return a invalid id
      }
    }
  }

  fn handle_packet( packet_id: Option<u8>, stream: &mut TcpStream, logger: &Arc<Logger>) -> Result<()>{
    let found_id = Server::packet_identifier(packet_id.unwrap());
    match found_id {
      control_type::CONNECT => { 
        println!("Connect packet received \n");
        logger.debug(format!("Peer mqtt connected: {}",stream.peer_addr()?));
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