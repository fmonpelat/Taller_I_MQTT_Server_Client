mod client;
use crate::client::Client;
use core::time;
use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::VariableHeader;
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet, ServerPacket};
use std::io::stdin;
use std::thread;

fn main() {
    let mut client = Client::new();

    println!("MQTT Client V1.0\n");

    loop {
        // creating a new default mqtt packet
        let packet: Packet<VariableHeader, Payload> = Packet::<VariableHeader, Payload>::new();
        // reading user input
        let user_input = user_input();

        if user_input.len() > 0 {
            match user_input[0].to_lowercase().as_ref() {
                "connect" => {
                    let host_str: Option<String> = user_input.get(1).and_then(|v| v.parse().ok());
                    let port_str: Option<String> = user_input.get(2).and_then(|v| v.parse().ok());
                    let username_str: Option<String> =
                        user_input.get(3).and_then(|v| v.parse().ok());
                    let password_str: Option<String> =
                        user_input.get(4).and_then(|v| v.parse().ok());

                    let mut host: String = String::from("");
                    let mut port: String = String::from("");
                    let mut username: String = String::from("");
                    let mut password: String = String::from("");

                    match host_str {
                        Some(_) => {
                            host = parser_str(user_input[1].to_string());
                        }
                        None => println!("non-existent host value"),
                    }

                    match port_str {
                        Some(_) => {
                            port = parser_str(user_input[2].to_string());
                        }
                        None => println!("non-existent port value"),
                    }

                    match username_str {
                        Some(_) => {
                            username = parser_str(user_input[3].to_string());
                        }
                        None => println!("non-existent username value"),
                    }

                    match password_str {
                        Some(_) => {
                            password = parser_str(user_input[4].to_string());
                        }
                        None => println!("non-existent password value"),
                    }

                    if !client.is_connected() {
                        // TODO: agregar en el client.connect(host, port, username, password) dentro del connect que seteen esos datos sobre el struct
                        client.connect(host, port, username, password);
                        let client_identifier = client.get_id_client();
                        println!("Connected to server with client id {}", client_identifier);
                        let packet = packet.connect(client_identifier.clone());
                        client.send(packet.value());
                        println!("send connect");
                        let mut i: usize = 0;
                        let conn_retries = client.get_connect_retries();
                        loop {
                            if client.is_connected() {
                                println!(
                                    "Connected to server with client id {}",
                                    client_identifier
                                );
                                break;
                            }
                            if i > conn_retries {
                                println!("Not connected to server");
                                break;
                            }
                            i += 1;
                            thread::sleep(time::Duration::from_millis(1000));
                            println!("waiting for connection ... retries: {}/{}", i, conn_retries);
                        }
                    } else {
                        println!("Already connected!");
                    }
                }
                "publish" => {
                    // send publish
                    if !client.is_connected() {
                        println!("Not connected to server, please connect first");
                        continue;
                    }

                    let dup_str: Option<String> = user_input.get(1).and_then(|v| v.parse().ok());
                    let qos_str: Option<String> = user_input.get(2).and_then(|v| v.parse().ok());
                    let retain_str: Option<String> = user_input.get(3).and_then(|v| v.parse().ok());
                    let topic_name_str: Option<String> =
                        user_input.get(4).and_then(|v| v.parse().ok());
                    let message_str: Option<String> =
                        user_input.get(5).and_then(|v| v.parse().ok());
                    let mut dup: u8 = 0;
                    let mut qos: u8 = 0;
                    let mut retain: u8 = 0;
                    let mut topic_name: String = String::from("");
                    let mut message: String = String::from("");

                    match dup_str {
                        Some(_) => {
                            dup = user_input[1].trim().parse().expect("wrong value!");
                        }
                        None => println!("non-existent dup value"),
                    }
                    match qos_str {
                        Some(_) => {
                            qos = user_input[2].trim().parse().expect("wrong value!");
                        }
                        None => println!("non-existent qos value"),
                    }
                    match retain_str {
                        Some(_) => {
                            retain = user_input[3].trim().parse().expect("wrong value!");
                        }
                        None => println!("non-existent retain value"),
                    }
                    match topic_name_str {
                        Some(_) => {
                            topic_name = user_input[4].clone();
                        }
                        None => println!("non-existent topic name value"),
                    }
                    match message_str {
                        Some(_) => {
                            message = user_input[5].clone();
                        }
                        None => println!("non-existent message value"),
                    }
                    println!("try publish");
                    let packet_identifier = client.get_packet_identifier();
                    let packet =
                        packet.publish(dup, qos, retain, packet_identifier, topic_name, message);
                    client.send(packet.value());
                }
                "pingreq" => {
                    // send publish
                    if !client.is_connected() {
                        println!("Not connected to server, please connect first");
                        continue;
                    }
                    let packet = packet.pingresp();
                    client.send(packet.value());
                    println!("send pingreq");
                }
                "disconnect" => {
                    if !client.is_connected() {
                        println!("Not connected to server, please connect first");
                        continue;
                    }
                    client.disconnect();
                }
                "test" => {
                    println!("test connection to localhost");
                    if !client.is_connected() {
                        // TODO: agregar en el client.connect(host, port, username, password) dentro del connect que seteen esos datos sobre el struct
                        client.connect(
                            "localhost".to_string(),
                            "3333".to_string(),
                            "test".to_string(),
                            "test".to_string(),
                        );
                        let client_identifier = client.get_id_client();
                        println!("Connected to server with client id {}", client_identifier);
                        let packet = packet.connect(client_identifier.clone());
                        client.send(packet.value());
                        println!("send connect");
                        let mut i: usize = 0;
                        let conn_retries = client.get_connect_retries();
                        loop {
                            if client.is_connected() {
                                println!(
                                    "Connected to server with client id {}",
                                    client_identifier
                                );
                                break;
                            }
                            if i > conn_retries {
                                println!("Not connected to server");
                                break;
                            }
                            i += 1;
                            thread::sleep(time::Duration::from_millis(1000));
                            println!("waiting for connection ... retries: {}/{}", i, conn_retries);
                        }
                    } else {
                        println!("Already connected!");
                    }
                }
                "test-connection" => {
                    if client.is_connected() {
                        println!(
                            "Connected to server with client id {}",
                            client.get_id_client()
                        );
                    } else {
                        println!("Not connected to server");
                    }
                }
                "exit" => {
                    break;
                }
                _ => {
                    println!("Message not understood: {:?}", user_input);
                    // print to stdout not understand
                }
            }
        } else {
            println!("Invalid command, to exit type: exit");
        }
    }
}

/// Opciones a ejecutar por el cliente.

pub fn user_input() -> Vec<String> {
    println!("Enter option...");
    let mut a_str = String::new();

    stdin().read_line(&mut a_str).expect("read error");
    let input_stdin: Vec<String> = a_str
        .split_whitespace()
        .map(|x| x.parse::<String>().expect("parse error"))
        .collect::<Vec<String>>();
    // println!("{:?}",input_stdin);
    input_stdin
}

pub fn parser_str(value: String) -> String {
    value.trim().parse().expect("wrong value!")
}

#[test]
    fn test_parser(){
        let vec = ["0","1","3"];
        assert_eq!("0",parser_str(vec[0].to_string()))
    }

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
