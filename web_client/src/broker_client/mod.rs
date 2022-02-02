use client::client::Client;
use std::time::Duration;
use std::thread;
use std::thread::{ sleep };
use std::sync::{ Mutex, Arc };
use std::io::Error;
use std::sync::mpsc::{ Receiver };

#[allow(dead_code)]
#[derive(Debug)]
pub struct BrokerClient {
    host: String,
    port: String,
    topic: String,
    conn_retries: usize, // number of retries to connect to broker
    connected: Arc<Mutex<bool>>,
    messages: Arc<Mutex<Vec<String>>>,
}

impl BrokerClient {
    pub fn new(host: String, port: String, topic: String, conn_retries: usize) -> Result<BrokerClient, Error> {
      let messages = Arc::new(Mutex::new(Vec::new())); // main structure to store messages
      let connected = Arc::new(Mutex::new(false)); // main structure to store connection status
      {
        let host = host.clone();
        let port = port.clone();
        let topic = topic.clone();
        let connected = connected.clone();
        let messages = messages.clone();

        let _handler = thread::Builder::new()
        .name("Thread Broker Client".to_string())
        .spawn(move || {
          let mut client = Client::new();
          client.set_keepalive_interval(20); // set keepalive interval to 20 seconds
          let messages_channel: Arc<Mutex<Receiver<String>>>;
          match client.connect(
              host,
              port,
              String::new(),
              String::new(),
              true,
              String::new(),
              String::new(),
          ) {
              Ok(rx_out) => {
                  println!("Connected to server");
                  *(connected.lock().unwrap()) = true;
                  messages_channel = rx_out;
              }
              Err(e) => {
                  panic!("Connection failed: {}", e);
              }
          };

          // wait until we are sucessfully connected
          let mut i: usize = 0;
          loop {
              if client.is_connected() {
                  client.subscribe(&topic);
                  break;
              }
              if i > conn_retries {
                  // print something as error on gui
                  client.disconnect();
                  panic!("Connection to broker failed.");
              }
              sleep(Duration::from_millis(2000));
              i += 1;
          }
          // get messages received by channel 
          if client.is_connected() {
            let message_receiver = messages_channel.lock().unwrap();
            loop {
                match message_receiver.recv() {
                    Ok(msg) => {
                      if msg.len() > 0 {
                        messages.lock().unwrap().push(msg.clone());
                      }
                    }
                    Err(e) => {
                        println!("Error receiving message from client channel: {}", e);
                    }
                }
            }
          }
        });
      }
      Ok(BrokerClient {
          host,
          port,
          topic,
          conn_retries,
          connected,
          messages,
      })
    }

    pub fn is_connected(&self) -> bool {
        *(self.connected.lock().unwrap())
    }

    pub fn get_last_messages(&self, count: usize) -> Vec<String> {
      let messages = self.messages.lock().unwrap();
      let mut result = Vec::new();
      for i in 0..count {
        if i < messages.len() {
          result.push(messages[messages.len() - i - 1].clone());
        }
      }
      result
    }

}
