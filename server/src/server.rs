use core::time;
use std::io::{Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::{thread};
use crate::logger::{Logger, Logging};

pub struct Server<'a> {
	server_address: &'a str, 
  server_port: &'a str,
	logger: Logger,   
  }

impl<'a> Server<'a> {
  pub fn new(server_address: &'a str, server_port: &'a str, file_source: & str) -> Self {
    Server{
      server_address,
      server_port,
	  logger: Logger::new(file_source),
    }
  }

  fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buff = [0_u8; 7]; // using 50 u8 buffer

    Ok(while match stream.read(&mut buff) {
        Ok(_size) => {
            //  send pong if ping msg is received
            match from_utf8(&buff) {
                Ok(packet) => {
                    match packet {
                        "Ping..." => {
                            println!("Ping received! \n");
                            if let Err(e) = stream.write_all(b"Pong...") {
                                println!("Client disconnect");
                                return Err(e) // Send client id when write_all fails
                            }
                        }
                        _ => println!("Not understood this packet: {}\n", packet),
                    }
                    thread::sleep(time::Duration::from_millis(2000));
                    true
                }
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            true
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {})
	}

  pub fn connect(&self) -> Result<()> {
	self.logger.debug("ready to binding".to_string());
    self.logger.info(format!("server address: {:?}",self.server_address.to_owned() + ":" + self.server_port));
    let listener = TcpListener::bind(self.server_address.to_owned() + ":" + self.server_port)?;
    // accept connections and process them, spawning a new thread for each one
    self.logger.debug("start binding".to_string());
    println!("Server listening on port {}", self.server_port);
    for stream in listener.incoming() {
        self.logger.info("start listening to clients".to_string());
        match stream {
            Ok(stream) => {
                self.logger.info(format!("New connection: {}",stream.peer_addr().unwrap()));
                thread::spawn(move || {
                    // connection succeeded
                    Server::<'a>::handle_client(stream)
                });
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