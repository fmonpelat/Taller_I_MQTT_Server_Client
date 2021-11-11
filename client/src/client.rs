use core::time;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::thread;
pub struct Client<'a> {
  server_host: &'a str, 
  server_port: &'a str,
    
}

impl<'a> Client<'a> {
  pub fn new(server_host: &'a str, server_port: &'a str) -> Self {
    Client{
      server_host,
      server_port,
    }
  }

  pub fn connect(&self) -> (){
    match TcpStream::connect(String::from(self.server_host) + ":" + self.server_port) {
      Ok(mut stream) => {
        println!("Successfully connected to server in port {}", self.server_port);

        let pingmsg = b"Ping...";
        stream.write_all(pingmsg).unwrap();

        let stream_arc = Arc::new(Mutex::new(stream));
        let _stream = Arc::clone(&stream_arc);

        let _handle_write = thread::spawn(move || loop {
            println!("Sent Ping, awaiting reply...");
            stream_arc.lock().unwrap().write_all(pingmsg).unwrap();

            thread::sleep(time::Duration::from_millis(2000));
        });

        let handle_read = thread::spawn(move || loop {
            let mut buff = [0_u8; 7];
            match _stream.lock().unwrap().read_exact(&mut buff) {
                Ok(_) => {
                    match from_utf8(&buff) {
                        Ok(packet) => {
                            println!("[client] buff:{}", from_utf8(&buff).unwrap());
                            match packet {
                                "Pong..." => {
                                    println!("Pong received!");
                                }
                                _ => println!("Unexpected reply: {}\n", packet),
                            }
                            true
                        }
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
            thread::sleep(time::Duration::from_millis(2000));
        });
        let _res = handle_read.join();
      }
      Err(e) => {
          println!("Failed to connect: {}", e);
      }
    };
    return;
  }

  // pub fn get_server_port(self)-> String{
  //   String::from(self.server_port)
  // }
}
trait Mqtt{
  fn publish();
  fn suscribe();
}

impl Mqtt for Client<'_>{
  fn publish() {
      
  }
  
  fn suscribe(){

  }
}
