use std::io::{Read, Write};
use std::iter;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
extern crate rand;
use mqtt_packet::mqtt_packet_service::header_packet::control_type;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{sync, thread};

use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::VariableHeader;
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet};

#[allow(dead_code)]
pub struct Client {
    server_host: String,
    server_port: String,
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
        Client {
            server_host: String::from(""),
            server_port: String::from(""),
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
        self.tx.lock().unwrap().send(value).unwrap_or_else(|_| println!("Cannot send packet"));
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

    pub fn connect(
        &mut self,
        host: String,
        port: String,
        username: String,
        password: String,
    ) -> () {
        // let Self {
        //   server_host,
        //   server_port,
        //   tx: _,
        //   rx,
        // } = self;

        self.server_host = host;
        self.server_port = port;
        self.username = username;
        self.password = password;
        match TcpStream::connect(self.server_host.to_string() + ":" + &self.server_port) {
            Ok(mut stream) => {
                println!(
                    "Successfully connected to server in port {}",
                    self.server_port
                );

                let packet = Packet::<VariableHeader, Payload>::new();
                let packet = packet.connect(self.client_identifier.clone());
                let msg: Vec<u8> = packet.value();

                match stream.write_all(&(msg)) {
                  Ok(v) => v,
                  Err(_) => println!("Can't write message. "),
              }

                let stream_arc = Arc::new(Mutex::new(stream));
                let _stream = Arc::clone(&stream_arc);

                // let rx = self.rx.clone();
                // let _handle_write = thread::Builder::new().name("Thread: write to stream".to_string())
                // .spawn( move ||
                //   loop {
                //     let guard = rx.lock().unwrap();
                //     match guard.recv() {
                //         Ok(msg) => {
                //             println!("Thread client write got a msg: {:?}", msg);
                //             // send message to stream
                //             stream_arc.lock().unwrap().write_all(&msg).unwrap();
                //             // Drop the `MutexGuard` to allow other threads to make use of rx
                //             drop(guard);
                //             //thread::sleep(Duration::from_millis(50));
                //         },
                //         Err(e) => {
                //             println!("Thread client write got a error: {:?}", e);
                //             break;
                //         }
                //     };
                //     // TODO: this sleep does not need to be here on production
                //     //thread::sleep(time::Duration::from_millis(30));
                //   }
                // );
                Client::handle_write(stream_arc, self.rx.clone());
                Client::handle_read(_stream, self.client_connection.clone());

                // let _res = handle_read.join();
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
                let guard = rx.lock().unwrap();
                
                match guard.recv() {
                    Ok(msg) => {
                        println!("Thread client write got a msg: {:?}", msg);
                        // send message to stream
                        stream.lock().unwrap().write_all(&msg).unwrap();
                        // Drop the `MutexGuard` to allow other threads to make use of rx
                        drop(guard);
                        //thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => {
                        println!("Thread client write got a error: {:?}", e);
                        break;
                    }
                };
                // TODO: this sleep does not need to be here on production
                //thread::sleep(time::Duration::from_millis(30));
            });
    }
    pub fn handle_read(stream: Arc<Mutex<TcpStream>>, client_connection: sync::Arc<AtomicBool>) {
        let _handle_read = thread::Builder::new()
            .name("Thread: read from stream".to_string())
            .spawn(move || loop {
                let mut buff = [0_u8; 1024];

                match stream.lock().unwrap().read(&mut buff) {
                    Ok(_) => {
                        if !buff.is_empty() {
                            match buff[0] {
                                control_type::CONNACK => {
                                    client_connection.store(true, Ordering::SeqCst);
                                    println!("Connack received!");
                                }
                                control_type::PUBACK => {
                                    println!("Puback received!");
                                }
                                control_type::SUBACK => {
                                    println!("Suback received!");
                                }
                                _ => println!("Unexpected reply: {:?}\n", buff),
                            }
                        }
                    }
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
                //thread::sleep(time::Duration::from_millis(60));
            });
    }

    pub fn get_id_client(&self) -> String {
        self.client_identifier.clone()
    }
}
