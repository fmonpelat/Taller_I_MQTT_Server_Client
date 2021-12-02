use core::time;
use std::io::{Read, Write};
use std::iter;
use std::net::{TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
extern crate rand;
use mqtt_packet::mqtt_packet_service::header_packet::control_type;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::VariableHeader;
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use std::{sync, thread};

#[allow(dead_code)]
pub struct Client {
    server_host: String,
    server_port: String,
    keepalive_rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    keepalive_tx: Arc<Mutex<Sender<Vec<u8>>>>,
    tx: Arc<Mutex<Sender<Vec<u8>>>>,
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    packet_identifier: u16,
    client_identifier: String,
    client_connection: sync::Arc<AtomicBool>,
    username: String,
    password: String,
    connect_retries: usize,
}

impl Client {
    pub fn new() -> Client {
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
        let rx = Arc::new(Mutex::new(rx));
        let tx = Arc::new(Mutex::new(tx));
        let mut rng = thread_rng();
        let client_identifier: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(10)
            .collect();
        let mut rng = rand::thread_rng();
        let packet_identifier: u16 = rng.gen();
        let connect_retries: usize = 10;
        let (keepalive_tx, keepalive_rx) = channel::<Vec<u8>>();
        Client {
            server_host: String::from(""),
            server_port: String::from(""),
            keepalive_rx: Arc::new(Mutex::new(keepalive_rx)),
            keepalive_tx: Arc::new(Mutex::new(keepalive_tx)),
            tx,
            rx,
            packet_identifier,
            client_identifier,
            client_connection: sync::Arc::new(AtomicBool::new(false)),
            username: String::from(""),
            password: String::from(""),
            connect_retries,
        }
    }

    pub fn get_connect_retries(&self) -> usize {
        self.connect_retries.clone()
    }

    pub fn is_connected(&self) -> bool {
        self.client_connection.load(Ordering::SeqCst).clone()
    }

    pub fn send(&self, value: Vec<u8>) {
        self.tx
            .lock()
            .unwrap()
            .send(value)
            .unwrap_or_else(|_| println!("Cannot send packet"));
    }

    pub fn get_packet_identifier(&self) -> u16 {
        self.packet_identifier.clone()
    }

    pub fn disconnect(&self) {
        let Self {
            server_host: _,
            server_port: _,
            tx,
            rx: _,
            packet_identifier: _,
            client_identifier: _,
            client_connection: _,
            username: _,
            password: _,
            connect_retries: _,
            keepalive_rx: _,
            keepalive_tx: _,
        } = self;
        let packet = Packet::<VariableHeader, Payload>::new();
        let packet = packet.disconnect();

        let msg = packet.value();
        if validate_msg(msg.clone()) {
            tx.lock().unwrap().send(msg).unwrap();
        } else {
            println!("can't send message: {:?}", msg);
        }

        fn validate_msg(msg: Vec<u8>) -> bool {
            let byte = msg[0];
            byte & 0xF0 == control_type::DISCONNECT
        }
    }

    pub fn keepalive(
        stream: Arc<Mutex<TcpStream>>,
        rx_: Arc<Mutex<Receiver<Vec<u8>>>>,
        keepalive_interval: usize,
    ) {
        // if keepalive is set to zero disable
        if keepalive_interval == 0 {
            return;
        }

        let mut keepalive_timer = Instant::now();

        let _handle = thread::Builder::new()
          .name("Thread: keepalive".to_string())
          .spawn(move || {
            let rx = &*rx_.lock().unwrap(); // safe to lock rx because it is only used in this thread
            loop {
              match rx.try_recv() {
                  Ok(msg) => {
                      if !msg.is_empty() {
                          keepalive_timer = Instant::now();
                      }
                  }
                  Err(e) => {
                      if e == std::sync::mpsc::TryRecvError::Empty {
                          if keepalive_timer.elapsed().as_secs() > (keepalive_interval as u64) / 3 {
                              keepalive_timer = Instant::now();
                              let packet = Packet::<VariableHeader, Payload>::new();
                              let packet = packet.pingreq();
                              let msg = packet.value();
                              println!("sending keepalive, locking stream");
                              stream
                                  .lock()
                                  .unwrap()
                                  .write_all(&msg)
                                  .expect("Could not write to stream sending pingreq");
                          }
                      }
                  }
              }

              if keepalive_timer.elapsed() > Duration::from_secs(keepalive_interval as u64) {
                  println!("keepalive timeout");
                  // call disconnect mqtt
                  break;
              }
            }
            thread::sleep(Duration::from_millis(10));
          });
    }

    pub fn connect(
        &mut self,
        host: String,
        port: String,
        username: String,
        password: String,
    ) -> () {
        self.server_host = host;
        self.server_port = port;
        self.username = username;
        self.password = password;
        match TcpStream::connect(self.server_host.to_string() + ":" + &self.server_port) {
            Ok(stream) => {
                println!(
                    "Successfully connected to server in port {}",
                    self.server_port
                );

                let packet = Packet::<VariableHeader, Payload>::new();
                let packet = packet.connect(self.client_identifier.clone());

                self.send(packet.value());

                let stream_ = Arc::new(Mutex::new(stream));

                Client::handle_read(
                    stream_.clone(),
                    self.client_connection.clone(),
                    self.keepalive_tx.clone(),
                    self.keepalive_rx.clone(),
                    4,
                );
                Client::handle_write(stream_.clone(), Arc::clone(&self.rx));
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        };
    }

    pub fn handle_write(stream: Arc<Mutex<TcpStream>>, rx: Arc<Mutex<Receiver<Vec<u8>>>>) {
        let _handle_write = thread::Builder::new()
            .name("Thread: write to stream".to_string())
            .spawn(move || loop {
                let rx_guard = rx.lock().unwrap();
                let mut stream_ = stream.lock().unwrap();

                match rx_guard.recv() {
                    Ok(msg) => {
                        println!("Thread client write got a msg: {:?}", msg);
                        // send message to stream
                        stream_.write_all(&msg).unwrap();
                        println!("Thread client successfully sended the message");
                    }
                    Err(e) => {
                        println!("Thread client write got an error: {:?}", e);
                        break;
                    }
                };
                drop(stream_);
                thread::sleep(Duration::from_millis(10));
            });
    }

    pub fn handle_read(
        stream: Arc<Mutex<TcpStream>>,
        client_connection: sync::Arc<AtomicBool>,
        keepalive_tx: Arc<Mutex<Sender<Vec<u8>>>>,
        keepalive_rx: Arc<Mutex<Receiver<Vec<u8>>>>,
        keepalive_interval: usize,
    ) {
        // let peer = stream.lock().unwrap().peer_addr().unwrap();
        let _handle_read = thread::Builder::new()
            .name("Thread: read from stream".to_string())
            .spawn(move || loop {
                let mut buff = [0_u8; 4098];

                let stream_lock = stream.lock().unwrap();
                let mut tpcstream = &*stream_lock;
                tpcstream
                    .set_read_timeout(Some(Duration::from_millis(100)))
                    .unwrap();
                match tpcstream.read(&mut buff) {
                    Ok(_size) => {
                        if _size > 0 {
                            match buff[0] {
                                control_type::CONNACK => {
                                    client_connection.store(true, Ordering::SeqCst);
                                    println!("Connack received! setting keepalive");
                                    Client::keepalive(stream.clone(), keepalive_rx.clone(), keepalive_interval);
                                }
                                control_type::PUBACK => {
                                    println!("Puback received!");
                                }
                                control_type::PINGRESP => {
                                    println!("Ping response received!");
                                    // write to channel tx of keepalive
                                    let keepalive_tx = &*keepalive_tx.lock().unwrap();
                                    keepalive_tx
                                        .send(vec![1])
                                        .expect("Could not send to keepalive channel");
                                }
                                control_type::SUBACK => {
                                    println!("Suback received!");
                                }
                                _ => println!("Unexpected reply: {:?}\n", buff),
                            }
                        }
                    }
                    Err(_error) => {
                        continue;
                        // println!("Failed to receive data, closing connection with server: {} error: {:?}",peer, error);
                        // // if we cant unwrap the mutex at this stage better panic
                        // stream_lock.shutdown(Shutdown::Both).unwrap();
                        // // drop(stream_lock);
                        // break
                    }
                }
                drop(stream_lock);
                thread::sleep(time::Duration::from_millis(10));
            });
    }

    pub fn get_id_client(&self) -> String {
        self.client_identifier.clone()
    }
}
