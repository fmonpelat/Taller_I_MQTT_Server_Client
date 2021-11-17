use core::time;
use std::net::{ TcpStream };
use std::io::{ Read, Write };
use std::sync::{ Arc, Mutex };

use std::thread;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;
//extern crate mqtt_packet;
//use mqtt_packet::mqtt_packet_service::{ Packet };

#[allow(dead_code)]
pub struct Client {
  server_host: String, 
  server_port: String,
  tx: Arc<Mutex<Sender<Vec<u8>>>>,
  rx: Arc<Mutex<Receiver<Vec<u8>>>>,
}

impl Client {
  pub fn new(server_host: String, server_port:  String ) -> Client {
    let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
    let rx = Arc::new(Mutex::new(rx));
    let tx = Arc::new(Mutex::new(tx));
    Client{
      server_host,
      server_port,
      tx,
      rx,
    }
  }

  pub fn publish(&self, _topic: String, _payload: String) {
    let Self { server_host: _, server_port: _, tx, rx: _ } = self;
    let msg = vec![0x30];

    tx.lock().unwrap()
    .send(msg).unwrap();
  }

  pub fn connect(&self) {
    // let Self {
    //   server_host,
    //   server_port,
    //   tx: _,
    //   rx,
    // } = self;

    match TcpStream::connect(self.server_host.to_string() + ":" + &self.server_port) {
      Ok(mut stream) => {
        println!("Successfully connected to server in port {}", self.server_port);

        let msg: Vec<u8> = vec![0x10]; // TODO : send connect packet value
        stream.write_all(&(msg)).unwrap();

        let stream_arc = Arc::new(Mutex::new(stream));
        let _stream = Arc::clone(&stream_arc);

        let rx = self.rx.clone();
        let _handle_write = thread::Builder::new().name("Thread: write to stream".to_string())
        .spawn( move || 
          loop {
            let guard = rx.lock().unwrap();
            match guard.recv() {
                Ok(msg) => {
                    println!("Thread client write got a msg: {:?}", msg);
                    // send message to stream
                    stream_arc.lock().unwrap().write_all(&msg).unwrap(); 
                    // Drop the `MutexGuard` to allow other threads to make use of rx
                    drop(guard);

                    thread::sleep(Duration::from_millis(50));
                },
                Err(e) => {
                    println!("Thread client write got a error: {:?}", e);
                    break;
                }
            };
            // TODO: this sleep does not need to be here on production
            thread::sleep(time::Duration::from_millis(30));
          }
        );
        
        let _handle_read = thread::Builder::new().name("Thread: read from stream".to_string())
        .spawn(move || loop {
            let mut buff: Vec<u8> = Vec::with_capacity(1024); 

            match _stream.lock().unwrap().read_exact(&mut buff) {
              Ok(_) => {
                if !buff.is_empty() {
                  println!("Thread client read got a msg: {:?}", buff);
                  println!("[client] buff:{:?}", buff);
                  match buff[0] {
                    0x20 => {println!("Connack received!") },
                      _ => println!("Unexpected reply: {:?}\n", buff),
                    }
                }  
              }
              Err(e) => {
                  println!("Failed to receive data: {}", e);
              }
            }
            thread::sleep(time::Duration::from_millis(60));
        });
        // let _res = handle_read.join();
      },
      Err(e) => {
          println!("Failed to connect: {}", e);
      }
    };
    
  }

}