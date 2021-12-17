use std::io::{Read, Write};
use std::iter;
use std::net::TcpStream;
use std::sync::{mpsc, Arc, Condvar, Mutex};
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
    keepalive_interval: u16,
    keepalive_pair: Arc<(Mutex<bool>, Condvar)>,
    packet_identifier: u16,
    client_identifier: String,
    client_connection: sync::Arc<AtomicBool>,
    last_packet_sent: Vec<u8>,
    username: String,
    password: String,
    connect_retries: usize,
    tx_out: Arc<Mutex<Sender<String>>>,
    rx_out: Arc<Mutex<Receiver<String>>>,
    tx_events_handler: Arc<Mutex<Sender<Vec<u8>>>>,
    rx_events_handler: Arc<Mutex<Receiver<Vec<u8>>>>,
}

impl Client {
    pub fn new() -> Client {
        let (tx, rx): (Sender<Vec<u8>>, Receiver<Vec<u8>>) = channel();
        let (tx_out, rx_out) = channel::<String>(); // channel to send outside messages
        let rx = Arc::new(Mutex::new(rx));
        let tx = Arc::new(Mutex::new(tx));
        let (tx_events_handler, rx_events_handler) = channel::<Vec<u8>>(); // channel to send events to act on
        let mut rng = thread_rng();
        let client_identifier: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(10)
            .collect();
        let mut rng = rand::thread_rng();
        let packet_identifier: u16 = rng.gen();
        let connect_retries: usize = 20;
        let keepalive_interval: u16 = 60;
        #[allow(clippy::mutex_atomic)]
        let keepalive_pair = Arc::new((Mutex::new(false), Condvar::new()));

        Client {
            server_host: String::from(""),
            server_port: String::from(""),
            tx,
            rx,
            keepalive_interval,
            keepalive_pair,
            packet_identifier,
            client_identifier,
            client_connection: sync::Arc::new(AtomicBool::new(false)),
            last_packet_sent: Vec::new(),
            username: String::from(""),
            password: String::from(""),
            connect_retries,
            tx_out: Arc::new(Mutex::new(tx_out)),
            rx_out: Arc::new(Mutex::new(rx_out)),
            tx_events_handler: Arc::new(Mutex::new(tx_events_handler)),
            rx_events_handler: Arc::new(Mutex::new(rx_events_handler)),
        }
    }

    #[allow(dead_code)]
    pub fn set_keepalive_interval(&mut self, keepalive_interval: u16) {
        self.keepalive_interval = keepalive_interval;
    }

    fn print_all(text: String, tx_out: sync::Arc<Mutex<Sender<String>>>) {
        println!("{}", text);
        match tx_out.lock() {
            Ok(tx_out_) => {
                tx_out_.send(text).unwrap();
            }
            Err(e) => {
                println!("{}", e);
            }
        }
        drop(tx_out);
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

    pub fn disconnect(&mut self) {
        let packet = Packet::<VariableHeader, Payload>::new();
        let packet = packet.disconnect();
        let pck_value = packet.value();
        self.last_packet_sent = pck_value.clone();
        self.send(pck_value);
        self.client_connection.store(false, Ordering::SeqCst);
    }

    pub fn disconnect_stream(tx: Arc<Mutex<Sender<Vec<u8>>>>) {
        let packet = Packet::<VariableHeader, Payload>::new();
        let packet = packet.disconnect();
        let tx = tx.lock().unwrap();
        tx.send(packet.value())
            .expect("Cannot send disconnect packet");
    }

    pub fn keepalive(
        tx: Arc<Mutex<Sender<Vec<u8>>>>,
        keepalive_interval: usize,
        pair: Arc<(Mutex<bool>, Condvar)>,
    ) {
        fn send_keepalive(tx: Arc<Mutex<Sender<Vec<u8>>>>) {
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.pingreq();
            let msg = packet.value();
            println!("sending keepalive");
            tx.lock()
                .unwrap()
                .send(msg)
                .expect("Could not write to stream sending pingreq");
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
                    send_keepalive(tx.clone());
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
                            Client::disconnect_stream(tx.clone());
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
                let pck_value = packet.value();
                self.last_packet_sent = pck_value.clone();
                self.send(pck_value);

                let stream_ = Arc::new(Mutex::new(stream));

                self.handle_events(
                    self.rx_events_handler.clone(),
                    self.tx.clone(),
                    self.client_connection.clone(),
                    self.keepalive_interval.into(),
                    self.keepalive_pair.clone(),
                    self.tx_out.clone(),
                );

                self.handle_io(stream_, self.rx.clone(), self.tx_events_handler.clone());

                Ok(self.rx_out.clone())
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                Err("Failed to connect")
            }
        }
    }

    fn handle_io(
        &mut self,
        stream: Arc<Mutex<TcpStream>>,
        rx: Arc<Mutex<Receiver<Vec<u8>>>>,
        tx_events_handler: Arc<Mutex<Sender<Vec<u8>>>>,
    ) {
        let _handle_io = thread::Builder::new()
            .name("Thread: IO Events stream".to_string())
            .spawn(move || loop {
                // lock stream
                let mut stream_ = stream.lock().unwrap();

                // Stream Writter
                // try to read from rx channel to send data to stream
                let rx_guard = rx.lock().unwrap();
                match rx_guard.try_recv() {
                    Ok(msg) => {
                        if msg[0] == control_type::DISCONNECT as u8 {
                            // disconnect
                            stream_.write_all(&msg).unwrap();
                            println!("Thread IO sended the disconnect message");
                            thread::sleep(Duration::from_secs(2));
                            match stream_.shutdown(std::net::Shutdown::Both) {
                                Ok(_) => {
                                    println!("Stream shutdown");
                                }
                                Err(_e) => {}
                            }
                            break;
                        }
                        println!("Thread IO got a msg to send with packet ID: {:?}", msg[0]);
                        // send message to stream
                        stream_.write_all(&msg).unwrap();
                        println!("Thread IO sended the message");
                    }
                    Err(e) => {
                        if e == mpsc::TryRecvError::Empty {
                            // println!("Thread IO channel is empty");
                        } else {
                            println!("Thread IO channel is closed");
                            break;
                        }
                    }
                };

                // Stream Reader
                {
                    // read from stream until timeout or disconnect
                    let mut buff = [0_u8; 4098];
                    let mut tpcstream = &*stream_;
                    tpcstream
                        .set_read_timeout(Some(Duration::from_millis(30)))
                        .unwrap();
                    match tpcstream.read(&mut buff) {
                        Ok(n) => {
                            if n > 0 {
                                // send message to event handler
                                tx_events_handler
                                    .lock()
                                    .unwrap()
                                    .send(buff.to_vec())
                                    .unwrap();
                                println!(
                                    "Thread IO got a msg to process event with Packet ID: {:?}",
                                    buff[0]
                                );
                            }
                        }
                        Err(_e) => {}
                    };
                }
                thread::yield_now();
            });
    }

    pub fn handle_events(
        &mut self,
        rx_events_handler: Arc<Mutex<Receiver<Vec<u8>>>>,
        tx: Arc<Mutex<Sender<Vec<u8>>>>,
        client_connection: Arc<AtomicBool>,
        keepalive_interval: usize,
        keepalive_pair: Arc<(Mutex<bool>, Condvar)>,
        tx_out: Arc<Mutex<Sender<String>>>,
    ) {
        let _handle_read = thread::Builder::new()
            .name("Thread: read from stream".to_string())
            .spawn(move || loop {
                // read from channel
                let rx = rx_events_handler.lock().unwrap();
                match rx.recv() {
                    Ok(msg) => {
                        if !msg.is_empty() {
                            match msg[0] & 0xF0 {
                                control_type::CONNACK => {
                                    client_connection.store(true, Ordering::SeqCst);
                                    Client::print_all(
                                        "Connack received! setting keepalive".to_string(),
                                        tx_out.clone(),
                                    );
                                    Client::keepalive(
                                        tx.clone(),
                                        keepalive_interval,
                                        keepalive_pair.clone(),
                                    );
                                }
                                control_type::PUBACK => {
                                    Client::print_all(
                                        "Puback received!".to_string(),
                                        tx_out.clone(),
                                    );
                                }
                                control_type::PUBLISH => {
                                    println!("Publish received!");
                                    let unvalue =
                                        Packet::<VariableHeaderPublish, PublishPayload>::unvalue(
                                            msg.to_vec(),
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
                                }
                                control_type::UNSUBACK => {
                                    println!("Unsuback received!");
                                    Client::print_all(
                                        "<-- Succesfully unsubscribed from topic".to_string(),
                                        tx_out.clone(),
                                    );
                                }
                                _ => {
                                    Client::print_all(
                                        format!("Unexpected reply: {:?}\n", msg),
                                        tx_out.clone(),
                                    );
                                }
                            }
                        }
                    }
                    Err(_error) => {
                        continue;
                    }
                }
                thread::yield_now();
            });
    }

    pub fn get_id_client(&self) -> String {
        self.client_identifier.clone()
    }

    #[allow(dead_code)]
    pub fn unsubscribe(&mut self, topic: &str) {
        let packet: Packet<VariableHeader, Payload> = Packet::<VariableHeader, Payload>::new();
        let packet_identifier = self.get_packet_identifier();
        let packet = packet.unsubscribe(packet_identifier, vec![topic.to_string()]);
        let pck_value = packet.value();
        self.last_packet_sent = pck_value.clone();
        Client::print_all(
            format!("--> unsubscribe topic: {} ", topic),
            self.tx_out.clone(),
        );
        self.send(pck_value);
    }

    #[allow(dead_code)]
    pub fn subscribe(&mut self, topic: &str) {
        let packet: Packet<VariableHeader, Payload> = Packet::<VariableHeader, Payload>::new();
        let packet_identifier = self.get_packet_identifier();
        let packet = packet.subscribe(packet_identifier, vec![String::from(topic)], vec![0]);
        let pck_value = packet.value();
        self.last_packet_sent = pck_value.clone();
        Client::print_all(
            format!("--> subscribe topic: {} ", topic),
            self.tx_out.clone(),
        );
        self.send(pck_value);
    }

    #[allow(dead_code)]
    pub fn publish(&mut self, qos: u8, dup: u8, retain: u8, topic_name: &str, message: &str) {
        let packet: Packet<VariableHeader, Payload> = Packet::<VariableHeader, Payload>::new();
        let packet_identifier = self.get_packet_identifier();
        let packet = packet.publish(
            dup,
            qos,
            retain,
            packet_identifier,
            topic_name.to_string(),
            message.to_string(),
        );
        let pck_value = packet.value();
        self.last_packet_sent = pck_value.clone();
        Client::print_all(
            format!("--> publish topic: {} value: {}", topic_name, message),
            self.tx_out.clone(),
        );
        self.send(pck_value);
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
