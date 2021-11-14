use core::time;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::sync::mpsc::{Sender,Receiver};
extern crate mqtt_packet;
use mqtt_packet::mqtt_packet::{Packet};

pub struct Client<'a> {
  server_host: &'a str, 
  server_port: &'a str,
  tx: Sender<Vec<u8>>,
  rx: Receiver<Vec<u8>>,    
}

impl<'a> Client<'a> {
  pub fn new(server_host: &'a str, server_port: &'a str ) -> Self {
    let (tx,rx) = mpsc::channel(); 
    Client{
      server_host,
      server_port,
      tx,
      rx,
    }
  }

  pub fn connect(&self) {
    
    match TcpStream::connect(String::from(self.server_host) + ":" + self.server_port) {
      Ok(mut stream) => {
        println!("Successfully connected to server in port {}", self.server_port);

        let msg: Vec<u8> = vec![0x10]; // TODO : send connect packet value
        stream.write_all(&(msg.clone())).unwrap();

        let stream_arc = Arc::new(Mutex::new(stream));
        let _stream = Arc::clone(&stream_arc);

        let _handle_write = thread::spawn(move || loop {
            let message = self.rx.try_recv();
            match message {
              Ok(msg) => {
                if msg.len() > 0 {
                  stream_arc.lock().unwrap().write_all(&msg).unwrap(); 
                }
              },
              Err(e) => {
                println!("Try rx channel received: {}",e);
              }
            }

            thread::sleep(time::Duration::from_millis(2000));
        });

        let handle_read = thread::spawn(move || loop {
            let mut buff: Vec<u8> = Vec::with_capacity(1024); 

            match _stream.lock().unwrap().read_exact(&mut buff) {
              
                Ok(_) => {
                      
                      println!("[client] buff:{:?}", buff);
                      match buff[0] {
                        0x20 => {println!("Pong received!") },
                            
                         _ => println!("Unexpected reply: {:?}\n", buff),
                        }
                      }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
            thread::sleep(time::Duration::from_millis(2000));
        });
        let _res = handle_read.join();
      },
      Err(e) => {
          println!("Failed to connect: {}", e);
      }
    };
    
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
