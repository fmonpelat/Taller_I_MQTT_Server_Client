use core::time;
use std::io::{Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::{Arc};
use std::{thread};
use crate::logger::{Logger, Logging};

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
            match buff[0] {
                0x10 => {  // TODO: use mqtt control mod packet type
                    println!("Connect packet received \n");
                    logger.info(format!("Peer mqtt connected: {}",stream.peer_addr().unwrap()));
                    if let Err(e) = stream.write_all(b"32") { // TODO: use mqtt control mod packet type
                        println!("Client disconnect");
                        return Err(e) // Send client id when write_all fails
                    }
                },
                _ => {
                    println!("Unknown command received! {:?}\n", buff);
                    logger.info(format!("Not understood this packet: {:?}\n", buff));
                }
            }
            thread::sleep(time::Duration::from_millis(2000));            
          }
        },
        Err(_) => {
          println!(
            "An error occurred, terminating connection with {}",
            stream.peer_addr().unwrap()
          );
          stream.shutdown(Shutdown::Both).unwrap();
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
                let peer = stream.peer_addr().unwrap();
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
}