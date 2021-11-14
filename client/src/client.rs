use core::time;
use std::net::{TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, MutexGuard, mpsc};
use std::thread;
//use std::sync::mpsc::{Sender,Receiver};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
//extern crate mqtt_packet;
use mqtt_packet::mqtt_packet_service::{Packet};

pub struct Client<'a> {
  server_host: &'a str, 
  server_port: &'a str,
  //tx: &'a Arc<Mutex<Sender<Vec<u8>>>>,
  //rx: &'a Arc<Mutex<Receiver<Vec<u8>>>>,
  tx: &'a SyncSender<Vec<u8>>,
  rx: &'a Receiver<Vec<u8>>, 
}

impl<'a> Client<'a> {
  pub fn new(server_host: &'a str, server_port: &'a str ) -> Client<'a>{
    let (tx,rx) = sync_channel(1); 
    //let tx1: &'a Arc<Mutex<Sender<Vec<u8>>>> = & Arc::new(Mutex::new(tx));
    //let rx1: &'a Arc<Mutex<Receiver<Vec<u8>>>> = & Arc::new(Mutex::new(rx));
    Client{
      server_host,
      server_port,
      tx: &tx,
      rx: &rx,
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

        let rx1= Arc::new(self.rx);
        let shared_rx = rx1.clone();

        let _tx1= &Arc::new(Mutex::new(self.tx));
        let _shared_tx1 = _tx1.clone();

        let _handle_write = thread::spawn( move || loop {
           let message = shared_rx.try_recv();
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
        drop(shared_rx);
        
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
