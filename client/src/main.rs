use std::net::{TcpStream};
use std::io::{Read, Write};
use std::str::from_utf8;
use std::{thread, time};

fn main() {
    let server_host = String::from("localhost");
    let server_port = "3333";

    match TcpStream::connect(server_host+":"+server_port) {
        Ok(mut stream) => {
            println!("Successfully connected to server in port {}", server_port);

            let msg = b"Ping..";

            stream.write(msg).unwrap();
            println!("Sent Ping, awaiting reply...");

            let mut data = [0 as u8; 6]; // using 6 byte buffer
            match stream.read_exact(&mut data) {
                Ok(_) => {
                    match from_utf8(&data) {
                        Ok(r_msg) => {
                            if r_msg == "Pong.." {
                                println!("Pong received!");
                                thread::sleep(time::Duration::from_millis(2000));
                                stream.write(msg).unwrap();
                                println!("Sent Ping, awaiting reply...");
                            } else {
                                println!("Unexpected reply: {}", r_msg);
                            }
                            true
                        },
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                },
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                }
            }
        },
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
