use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::str;


fn handle_client(mut stream: TcpStream) {
    let mut data = [0 as u8; 50]; // using 50 byte buffer
    while match stream.read(&mut data) {
        Ok(_size) => {
            //  send pong if ping msg is received
            match str::from_utf8(&data) {
                Ok(v) => {
                    if v == "Ping.." {
                        println!("Ping received! v = {}\n", v);
                        stream.write(b"Pong..").unwrap();
                    }
                },
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            true
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {}
}

fn main() {
    let server_address = String::from("0.0.0.0");
    let server_port = "3333";
    let listener = TcpListener::bind(server_address+":"+server_port).unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port {}", server_port);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move|| {
                    // connection succeeded
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                /* connection failed */
            }
        }
    }
    // close the socket server
    drop(listener);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_server() {
        assert_eq!(1, 1)
    }
}
