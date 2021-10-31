use std::io::{Read, Write, Result, BufWriter};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::{thread, time};
mod logger;
use logger::{Logging};
use std::fs::{File};

fn handle_client(mut stream: TcpStream) -> Result<()> {
    let mut buff = [0_u8; 7]; // using 50 u8 buffer

    Ok(while match stream.read(&mut buff) {
        Ok(_size) => {
            //  send pong if ping msg is received
            match from_utf8(&buff) {
                Ok(packet) => {
                    match packet {
                        "Ping..." => {
                            println!("Ping received! \n");
                            if let Err(e) = stream.write_all(b"Pong...") {
                                println!("Client disconnect");
                                return Err(e) // Send client id when write_all fails
                            }
                        }
                        _ => println!("Not understood this packet: {}\n", packet),
                    }
                    thread::sleep(time::Duration::from_millis(2000));
                    true
                }
                Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
            };
            true
        }
        Err(_) => {
            println!(
                "An error occurred, terminating connection with {}",
                stream.peer_addr().unwrap()
            );
            stream.shutdown(Shutdown::Both).unwrap();
            false
        }
    } {})
}

fn main() {
    let file_name = "log.txt".to_string();
    let source_log = "../".to_string();
    let logfile: Arc<Mutex<BufWriter<File>>> = Logging::new(&file_name,&source_log);
    let server_address = String::from("0.0.0.0");
    let server_port = "3333";
    let listener = TcpListener::bind(server_address + ":" + server_port).unwrap();
    // accept connections and process them, spawning a new thread for each one
    logfile.log("start binding".to_string());
    println!("Server listening on port {}", server_port);
    for stream in listener.incoming() {
        logfile.log("start listening to clients".to_string());
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                thread::spawn(move || {
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
