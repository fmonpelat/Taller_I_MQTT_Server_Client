use core::time;
use std::io::{Read, Write};
use std::iter;
use std::net::TcpStream;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;
extern crate rand;
use mqtt_packet::mqtt_packet_service::header_packet::control_type;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

use mqtt_packet::mqtt_packet_service::payload_packet::{Payload, PublishPayload};
use mqtt_packet::mqtt_packet_service::variable_header_packet::{
    VariableHeader, VariableHeaderPublish,
};
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use std::{sync, thread};

#[allow(dead_code)]
pub struct Client {
    server_host: String,
    server_port: String,
    tx: Arc<Mutex<Sender<Vec<u8>>>>,
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    keepalive_pair: Arc<(Mutex<bool>, Condvar)>,
    packet_identifier: u16,
    client_identifier: String,
    client_connection: sync::Arc<AtomicBool>,
    username: String,
    password: String,
    connect_retries: usize,
    tx_out: Arc<Mutex<Sender<String>>>,
    rx_out: Arc<Mutex<Receiver<String>>>,
}

impl Client {
    pub fn new() -> Client {
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
        let (tx_out, rx_out) = channel::<String>(); // channel to send outside messages
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
        let connect_retries: usize = 20;
        let keepalive_pair = Arc::new((Mutex::new(false), Condvar::new()));

        Client {
            server_host: String::from(""),
            server_port: String::from(""),
            tx,
            rx,
            keepalive_pair,
            packet_identifier,
            client_identifier,
            client_connection: sync::Arc::new(AtomicBool::new(false)),
            username: String::from(""),
            password: String::from(""),
            connect_retries,
            tx_out: Arc::new(Mutex::new(tx_out)),
            rx_out: Arc::new(Mutex::new(rx_out)),
        }
    }

    fn print_all(text: String, tx_out: sync::Arc<Mutex<Sender<String>>>) {
        println!("{}", text);
        match tx_out.lock() {
            Ok(tx_out) => {
                tx_out.send(text).unwrap();
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    pub fn get_connect_retries(&self) -> usize {
        self.connect_retries
    }

    pub fn is_connected(&self) -> bool {
        self.client_connection.load(Ordering::SeqCst)
    }

    pub fn send(&self, value: Vec<u8>) {
        self.tx
            .lock()
            .unwrap()
            .send(value)
            .unwrap_or_else(|_| println!("Cannot send packet"));
    }

    pub fn get_packet_identifier(&self) -> u16 {
        self.packet_identifier
    }

    pub fn disconnect(&self) {
        let packet = Packet::<VariableHeader, Payload>::new();
        let packet = packet.disconnect();
        self.tx
            .lock()
            .unwrap()
            .send(packet.value())
            .expect("Cannot send packet");
        self.client_connection.store(false, Ordering::SeqCst);
    }

    pub fn disconnect_stream(stream: Arc<Mutex<TcpStream>>) {
        let packet = Packet::<VariableHeader, Payload>::new();
        let packet = packet.disconnect();
        let mut stream = stream.lock().unwrap();
        stream
            .write_all(&packet.value())
            .expect("Cannot send packet");
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    }

    pub fn keepalive(
        stream: Arc<Mutex<TcpStream>>,
        keepalive_interval: usize,
        pair: Arc<(Mutex<bool>, Condvar)>,
    ) {
        fn send_keepalive(stream: Arc<Mutex<TcpStream>>) {
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.pingreq();
            let msg = packet.value();
            println!("sending keepalive");
            stream
                .lock()
                .unwrap()
                .write_all(&msg)
                .expect("Could not write to stream sending pingreq");
            drop(stream);
        }

        // if keepalive is set to zero disable
        if keepalive_interval == 0 {
            return;
        }

        let mut keepalive_timer = Instant::now();

        let _handle = thread::Builder::new()
            .name("Thread: keepalive".to_string())
            .spawn(move || {
                let (lock, cvar) = &*pair;
                loop {
                    send_keepalive(stream.clone());
                    let received = lock.lock().unwrap();
                    let result = cvar
                        .wait_timeout(received, Duration::from_secs(keepalive_interval as u64))
                        .unwrap();
                    let mut received = result.0;
                    if *received {
                        // send next keepalive
                        keepalive_timer = Instant::now();
                        *received = false;
                        println!("received keepalive response from server");
                    } else {
                        // check if keepalive has timed out
                        if keepalive_timer.elapsed().as_secs() >= keepalive_interval as u64 {
                            println!("keepalive timed out");
                            Client::disconnect_stream(stream.clone());
                            break;
                        }
                    }
                    thread::sleep(Duration::from_secs(keepalive_interval as u64));
                }
            });
    }

    pub fn connect(
        &mut self,
        host: String,
        port: String,
        username: String,
        password: String,
    ) -> Result<Arc<Mutex<Receiver<String>>>, &str> {
        self.server_host = host;
        self.server_port = port;
        self.username = username;
        self.password = password;
        let keepalive_interval = 20;
        match TcpStream::connect(self.server_host.to_string() + ":" + &self.server_port) {
            Ok(stream) => {
                Client::print_all(
                    format!(
                        "Successfully connected to server in port {}",
                        self.server_port
                    ),
                    self.tx_out.clone(),
                );

                let mut packet = Packet::<VariableHeader, Payload>::new();

                if self.username.is_empty() || self.password.is_empty() {
                    println!(
                        "No username provided, skipping username/password for client {}",
                        self.client_identifier
                    );
                    packet = packet.connect(self.client_identifier.clone());
                } else {
                    println!(
                        "Connecting with credentials for client {}",
                        self.client_identifier
                    );
                    packet = packet.connect_with_credentials(
                        self.client_identifier.clone(),
                        self.username.clone(),
                        self.password.clone(),
                    );
                }
                self.send(packet.value());

                let stream_ = Arc::new(Mutex::new(stream));

                Client::handle_read(
                    stream_.clone(),
                    self.client_connection.clone(),
                    keepalive_interval,
                    self.keepalive_pair.clone(),
                    self.tx_out.clone(),
                );
                Client::handle_write(stream_, Arc::clone(&self.rx));
                Ok(self.rx_out.clone())
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                Err("Failed to connect")
            }
        }
    }

    pub fn handle_write(stream: Arc<Mutex<TcpStream>>, rx: Arc<Mutex<Receiver<Vec<u8>>>>) {
        let _handle_write = thread::Builder::new()
            .name("Thread: write to stream".to_string())
            .spawn(move || loop {
                let rx_guard = rx.lock().unwrap();

                match rx_guard.recv() {
                    Ok(msg) => {
                        println!("Thread client write got a msg: {:?}", msg);
                        // send message to stream
                        let mut stream_ = stream.lock().unwrap();
                        stream_.write_all(&msg).unwrap();
                        println!("Thread client successfully sended the message");
                    }
                    Err(e) => {
                        println!("Thread client write got an error: {:?}", e);
                        break;
                    }
                };
                thread::sleep(Duration::from_millis(10));
            });
    }

    pub fn handle_read(
        stream: Arc<Mutex<TcpStream>>,
        client_connection: sync::Arc<AtomicBool>,
        keepalive_interval: usize,
        keepalive_pair: Arc<(Mutex<bool>, Condvar)>,
        tx_out: sync::Arc<Mutex<Sender<String>>>,
    ) {
        // let peer = stream.lock().unwrap().peer_addr().unwrap();
        let _handle_read = thread::Builder::new()
            .name("Thread: read from stream".to_string())
            .spawn(move || loop {
                let mut buff = [0_u8; 4098];

                let stream_lock = stream.lock().unwrap();
                let mut tpcstream = &*stream_lock;
                tpcstream
                    .set_read_timeout(Some(Duration::from_millis(30)))
                    .unwrap();
                match tpcstream.read(&mut buff) {
                    Ok(_size) => {
                        if _size > 0 {
                            match buff[0] & 0xF0 {
                                control_type::CONNACK => {
                                    client_connection.store(true, Ordering::SeqCst);
                                    // println!("Connack received! setting keepalive");
                                    Client::print_all(
                                        "Connack received! setting keepalive".to_string(),
                                        tx_out.clone(),
                                    );
                                    Client::keepalive(
                                        stream.clone(),
                                        keepalive_interval,
                                        keepalive_pair.clone(),
                                    );
                                }
                                control_type::PUBACK => {
                                    // println!("Puback received!");
                                    Client::print_all(
                                        "Puback received!".to_string(),
                                        tx_out.clone(),
                                    );
                                }
                                control_type::PUBLISH => {
                                    println!("Publish received!");
                                    let unvalue =
                                        Packet::<VariableHeaderPublish, PublishPayload>::unvalue(
                                            buff.to_vec(),
                                        );
                                    Client::print_all(
                                        format!(
                                            "<-- publish topic: {} value: {}",
                                            String::from_utf8_lossy(
                                                &unvalue.variable_header.topic_name
                                            ),
                                            unvalue.payload.message
                                        ),
                                        tx_out.clone(),
                                    );
                                    // println!(
                                    //     "<-- publish topic: {} value: {}",
                                    //     String::from_utf8_lossy(
                                    //         &unvalue.variable_header.topic_name
                                    //     ),
                                    //     unvalue.payload.message
                                    // );
                                }
                                control_type::PINGRESP => {
                                    let (lock, cvar) = &*keepalive_pair;
                                    let mut received = lock.lock().unwrap();
                                    *received = true;
                                    // We notify the condvar that the value has changed.
                                    cvar.notify_one();
                                }
                                control_type::SUBACK => {
                                    println!("Suback received!");
                                    Client::print_all(
                                        "<-- Succesfully subscribed to topic".to_string(),
                                        tx_out.clone(),
                                    );
                                    // println!("<-- Succesfully subscribed to topic");
                                }
                                control_type::UNSUBACK => {
                                    println!("Unsuback received!");
                                    Client::print_all(
                                        "<-- Succesfully unsubscribed from topic".to_string(),
                                        tx_out.clone(),
                                    );
                                    // println!("<-- Succesfully unsubscribed from topic");
                                }
                                _ => {
                                    // println!("Unexpected reply: {:?}\n", buff);
                                    Client::print_all(
                                        format!("Unexpected reply: {:?}\n", buff),
                                        tx_out.clone(),
                                    );
                                }
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

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
