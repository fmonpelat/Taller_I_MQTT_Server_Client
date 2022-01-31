use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::fs;
use web_client::ThreadPool;
use web_client::file_loader;
// use web_client::logger;
use web_client::http::parse_request_line;


fn main() {
    let config = file_loader::load_contents("src/config.yaml");
    let host = config
        .get("host")
        .unwrap_or_else(|| panic!("Cannot found host in config"));
    let port = config
        .get("port")
        .unwrap_or_else(|| panic!("Cannot found port in config"));
    let _broker_host = config
        .get("broker_host")
        .unwrap_or_else(|| panic!("Cannot found broker_host in config"));
    let _broker_port = config
        .get("broker_port")
        .unwrap_or_else(|| panic!("Cannot found broker_port in config"));
    let _topic = config
        .get("topic")
        .unwrap_or_else(|| panic!("Cannot found topic in config"));
   


    // connect to broker client and subscribe to topic
    // BrokerClient must connect to broker subscribe to topic and receive messages and save it to own structure
    // let mut broker_client = BrokerClient::new(_broker_host, _broker_port); // connects to broker and creates a thread to handle IO messages
    // broker_client.subscribe(_topic);  // subscribes to topic
    // broker_client.get_last_messages(10); // returns vector of string messages


    println!("Starting http server on {}:{}", host.clone(), port.clone());
    let listener = TcpListener::bind(host.to_owned() + ":" + port).unwrap();
    let pool = ThreadPool::new(10);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) -> () {

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
            println!("\n{}", request);
            contents = fs::read_to_string("bin/index.html").unwrap();
            response = format!("{}{}", "HTTP/1.1 200 OK\r\n\r\n", contents);
            println!("Response: {}\n", &response);
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

