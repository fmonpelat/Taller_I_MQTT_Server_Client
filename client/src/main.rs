use std::io::{Read, Write};
use std::net::TcpStream;
use std::str::from_utf8;
use std::sync::{Arc, Mutex};
use std::{thread, time};

fn main() {
    let server_host = String::from("localhost");
    let server_port = "3333";

    match TcpStream::connect(server_host + ":" + server_port) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port {}", server_port);

            let pingmsg = b"Ping...";
            stream.write(pingmsg).unwrap();

            let stream_arc = Arc::new(Mutex::new(stream));
            let _stream = Arc::clone(&stream_arc);

            let _handle_write = thread::spawn(move || loop {
                println!("Sent Ping, awaiting reply...");
                stream_arc.lock().unwrap().write(pingmsg).unwrap();

                thread::sleep(time::Duration::from_millis(2000));
            });

            let handle_read = thread::spawn(move || loop {
                let mut buff = [0 as u8; 7];
                match _stream.lock().unwrap().read_exact(&mut buff) {
                    Ok(_) => {
                        match from_utf8(&buff) {
                            Ok(packet) => {
                                println!("[client] buff:{}", from_utf8(&buff).unwrap());
                                match packet {
                                    "Pong..." => {
                                        println!("Pong received!");
                                    }
                                    _ => println!("Unexpected reply: {}\n", packet),
                                }
                                true
                            }
                            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                        };
                    }
                    Err(e) => {
                        println!("Failed to receive data: {}", e);
                    }
                }
                thread::sleep(time::Duration::from_millis(2000));
            });
            let _res = handle_read.join();
        }
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Client terminated.");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
