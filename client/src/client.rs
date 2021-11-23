use core::time;
use std::net::{ TcpStream };
use std::io::{ Read, Write };
use std::sync::{ Arc, Mutex };
use std::iter;
extern crate rand;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use std::{sync, thread};
use std::time as timer;
use std::sync::atomic::{AtomicBool, Ordering};

use mqtt_packet::mqtt_packet_service::{Packet};
use mqtt_packet::mqtt_packet_service::variable_header_packet::{VariableHeader};
use mqtt_packet::mqtt_packet_service::payload_packet::{Payload};

#[allow(dead_code)]
pub struct Client {
  server_host: String, 
  server_port: String,
  tx: Arc<Mutex<Sender<Vec<u8>>>>,
  rx: Arc<Mutex<Receiver<Vec<u8>>>>,
  id_client: String,
}

pub struct ClientConected {
  handle: Option<thread::JoinHandle<()>>,
  pub isConnected: sync::Arc<AtomicBool>,
}

impl ClientConected {
  pub fn new() -> ClientConected {
    ClientConected {
          handle: None,
          isConnected: sync::Arc::new(AtomicBool::new(false)),
      }
  }

  pub fn startConnection(&mut self) -> ()
  {
      self.isConnected.store(true, Ordering::SeqCst);

      let isConnected = self.isConnected.clone();

      self.handle = Some(thread::spawn(move || {
          //let mut funtion = funtion;
          while isConnected.load(Ordering::SeqCst) {
            //funtion();
            thread::sleep(timer::Duration::from_millis(10));
          }
      }));
  }

  pub fn stopConnection(&mut self) {
      self.isConnected.store(false, Ordering::SeqCst);
      self.handle
          .take().expect("Non-running thread")
          .join().expect("Could not join");
  }
}

impl Client {
  pub fn new(server_host: String, server_port:  String ) -> Client {
    let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
    let rx = Arc::new(Mutex::new(rx));
    let tx = Arc::new(Mutex::new(tx));
    let mut rng = thread_rng();
    let id_client: String = iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .map(char::from)
        .take(10)
        .collect();
    Client{
      server_host,
      server_port,
      tx,
      rx,
      id_client,
    }
  }


  pub fn publish(&self, _topic: String, _payload: String) {
    let Self { server_host: _, server_port: _, tx, rx: _ , id_client: _} = self;
    let msg = vec![0x30];

    let packet = Packet::<VariableHeader, Payload>::new();
    //let packet = packet.publish(dup, qos, retain, topic_name, payload);

    if validate_msg(msg.clone()) {
      tx.lock().unwrap()
      .send(msg).unwrap();
    } else {
      println!("can't send message: {:?}", msg);
    }

    fn validate_msg(msg: Vec<u8>) -> bool {
      let byte = msg[0];
      byte & 0xF0 == 0x10
    }
  }

  pub fn send(&self, value:Vec<u8>){
    
    if validate_msg(value.clone()) {
      self.tx.lock().unwrap()
      .send(value).unwrap();
    } else {
      println!("can't send message: {:?}", value);
    }

    fn validate_msg(msg: Vec<u8>) -> bool {
      let byte = msg[1];
      byte & 0xF0 == 0x10
    }
  }

  pub fn connect(& self) -> ClientConected {
    // let Self {
    //   server_host,
    //   server_port,
    //   tx: _,
    //   rx,
    // } = self;

    let mut clientConnected = ClientConected::new();
    match TcpStream::connect(self.server_host.to_string() + ":" + &self.server_port) {
      Ok(mut stream) => {
        println!("Successfully connected to server in port {}", self.server_port);

        let packet = Packet::<VariableHeader, Payload>::new();
        let packet = packet.connect(self.client_id.clone());
        let msg: Vec<u8> = packet.value(); 

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
                    clientConnected.startConnection();
                    thread::sleep(Duration::from_millis(50));
                },
                Err(e) => {
                    println!("Thread client write got a error: {:?}", e);
                    clientConnected.stopConnection();
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
    clientConnected
  }

  pub fn get_id_client(&self) -> String {
    self.id_client.clone()
  }
}