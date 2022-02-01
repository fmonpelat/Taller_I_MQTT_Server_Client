use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use web_client::{ ThreadPool, file_loader::load_contents, http::parse_request_line, broker_client::BrokerClient };
use std::sync::{ Mutex, Arc };


fn main() {
    let config = load_contents("src/config.yaml");
    let host = config
        .get("host")
        .unwrap_or_else(|| panic!("Cannot found host in config"));
    let port = config
        .get("port")
        .unwrap_or_else(|| panic!("Cannot found port in config"));
    let broker_host = config
        .get("broker_host")
        .unwrap_or_else(|| panic!("Cannot found broker_host in config"));
    let broker_port = config
        .get("broker_port")
        .unwrap_or_else(|| panic!("Cannot found broker_port in config"));
    let topic = config
        .get("topic")
        .unwrap_or_else(|| panic!("Cannot found topic in config"));
    let default_broker_conn_retries = String::from("10");
    let broker_conn_retries: usize = config
        .get("broker_conn_retries")
        .unwrap_or_else(|| &default_broker_conn_retries).parse().unwrap();
    


    // connect to broker client and subscribe to topic
    let messages: Vec<String>;
    // connects to broker and creates a thread to handle IO messages
    let broker_client = BrokerClient::new(broker_host.clone(), broker_port.clone(), topic.clone(), broker_conn_retries).unwrap_or_else(|e| panic!("Connection to broker failed: {}", e));
    // let messages: Vec<String> = broker_client.get_last_messages(10); // returns vector of string messages

    let broker_client: Arc<Mutex<BrokerClient>> = Arc::new(Mutex::new(broker_client));

    println!("Starting http server on {}:{}", host.clone(), port.clone());
    let listener = TcpListener::bind(host.to_owned() + ":" + &port).unwrap();
    let pool = ThreadPool::new(10);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let broker_client = broker_client.clone();
        pool.execute(|| {
            handle_connection(stream, broker_client);
        });
    }
}

fn handle_connection(mut stream: TcpStream, broker_client: Arc<Mutex<BrokerClient>>) -> () {

    // 1024 bytes is enough for a toy HTTP server
    let mut buffer = [0; 1024];
 
     // writes stream into buffer
     stream.read(&mut buffer).unwrap();
 
     let request = String::from_utf8_lossy(&buffer[..]);
     let request_line = request.lines().next().unwrap();

    let mut contents = String::new();
    let mut response = String::new();
    match parse_request_line(&request_line) {
        Ok(request) => {
            // println!("\n{}", request);
            match broker_client.lock() {
                Ok(client) => {
                    let connected = client.is_connected();
                    if connected == false {
                        contents = fs::read_to_string("src/bin/502.html").unwrap();
                        response = format!("{}{}", "HTTP/1.1 502 Bad Gateway\r\n\r\n", contents);
                    } else {
                        let messages = client.get_last_messages(10);
                        contents = fs::read_to_string("src/bin/index.html").unwrap();
                        let mut all_html_styled_messages = String::new();
                        let messages_placeholder = "<div class=\"messages\">".to_string();
                        for message in messages {
                            all_html_styled_messages.push_str(&format!("<p>{}</p>", message));
                        }
                        let contents = contents.replace(&messages_placeholder.clone(), &(messages_placeholder + &all_html_styled_messages));

                        response = format!("{}{}", "HTTP/1.1 200 OK\r\n\r\n", contents);
                    }
                },
                Err(e) => {
                    println!("this is the error{}", e);
                }
            }
            // println!("Response: {}\n", &response);
        }
        Err(_) => {
            contents = fs::read_to_string("bin/404.html").unwrap();
            response = format!("{}{}", "HTTP/1.1 404 NOT FOUND\r\n\r\n", contents);
            println!("Request invalid: {}", &request_line)
        },
    }

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

    ()
}

