mod client;
use crate::client::Client;
use core::time;
use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::VariableHeader;
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet};
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

        if !user_input.is_empty() {
            match user_input[0].to_lowercase().as_ref() {
                "connect" => {
                    let host_str: Option<String> = user_input.get(1).and_then(|v| v.parse().ok());
                    let port_str: Option<String> = user_input.get(2).and_then(|v| v.parse().ok());
                    let username_str: Option<String> =
                        user_input.get(3).and_then(|v| v.parse().ok());
                    let password_str: Option<String> =
                        user_input.get(4).and_then(|v| v.parse().ok());
                    let will_topic_str: Option<String> =
                        user_input.get(5).and_then(|v| v.parse().ok());
                    let will_message_str: Option<String> =
                        user_input.get(6).and_then(|v| v.parse().ok());

                    let host: String;
                    let port: String;
                    let mut username: String = String::new();
                    let mut password: String = String::new();
                    let mut will_topic: String = String::new();
                    let mut will_message: String = String::new();

                    match host_str {
                        Some(_) => {
                            host = parser_str(user_input[1].to_string());
                        }
                        None => {
                            println!("Non-existent host value");
                            continue;
                        }
                    }

                    match port_str {
                        Some(_) => {
                            port = parser_str(user_input[2].to_string());
                        }
                        None => {
                            println!("Non-existent port value");
                            continue;
                        }
                    }

                    match username_str {
                        Some(_) => {
                            username = parser_str(user_input[3].to_string());
                        }
                        None => println!("Non-existent username value"),
                    }

                    match password_str {
                        Some(_) => {
                            password = parser_str(user_input[4].to_string());
                        }
                        None => println!("Non-existent password value"),
                    }

                    match will_topic_str {
                        Some(_) => {
                            will_topic = parser_str(user_input[5].to_string());
                        }
                        None => println!("Non-existent will_topic value"),
                    }

                    match will_message_str {
                        Some(_) => {
                            will_message = parser_str(user_input[6].to_string());
                        }
                        None => println!("Non-existent will_message value"),
                    }

                    if !client.is_connected() {
                        match client.connect(
                            host.clone(),
                            port.clone(),
                            username.clone(),
                            password.clone(),
                            true,
                            will_topic.clone(),
                            will_message.clone(),
                        ) {
                            Ok(_) => {
                                println!(
                                    "--> connect to server with host: {} port: {}",
                                    host, port
                                );
                            }
                            Err(_) => {
                                println!("--> Error connecting with host: {} port: {}", host, port);
                                continue;
                            }
                        };

                        let client_identifier = client.get_id_client();
                        println!("--> Trying to connect with client id {}", client_identifier);
                        let mut i: usize = 0;
                        let conn_retries = client.get_connect_retries();
                        loop {
                            if client.is_connected() {
                                println!(
                                    "--> Connected to server with client id {}",
                                    client_identifier
                                );
                                break;
                            }
                            if i >= conn_retries {
                                println!("--> Not connected to server");
                                break;
                            }
                            i += 1;
                            thread::sleep(time::Duration::from_millis(2000));
                            println!("waiting for connection ... retries: {}/{}", i, conn_retries);
                        }
                    } else {
                        println!("--> Already connected!");
                        continue;
                    }
                }

                "publish" => {
                    // send publish
                    if !client.is_connected() {
                        println!(" <-- Not connected to server. Please connect first");
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
                            dup = user_input[1].trim().parse().expect("Wrong value!");
                        }
                        None => println!("Non-existent dup value"),
                    }
                    match qos_str {
                        Some(_) => {
                            qos = user_input[2].trim().parse().expect("Wrong value!");
                        }
                        None => println!("Non-existent qos value"),
                    }
                    match retain_str {
                        Some(_) => {
                            retain = user_input[3].trim().parse().expect("wrong value!");
                        }
                        None => println!("Non-existent retain value"),
                    }
                    match topic_name_str {
                        Some(_) => {
                            topic_name = user_input[4].clone();
                        }
                        None => println!("Non-existent topic name value"),
                    }
                    match message_str {
                        Some(_) => {
                            message = user_input[5].clone();
                        }
                        None => println!("Non-existent message value"),
                    }
                    let packet_identifier = client.get_packet_identifier();
                    let packet = packet.publish(
                        dup,
                        qos,
                        retain,
                        packet_identifier,
                        topic_name.clone(),
                        message.clone(),
                    );
                    println!("--> publish topic: {} value: {}", topic_name, message);
                    client.send(packet.value());
                }

                "disconnect" => {
                    if !client.is_connected() {
                        println!(" <-- Not connected to server. Please connect first");
                        continue;
                    }
                    client.disconnect();
                    println!("--> Disconnected from server.");
                }

                "subscribe" => {
                    // subscribe qos topic_name
                    // send subscribe
                    if !client.is_connected() {
                        println!(" <-- Not connected to server, please connect first");
                        continue;
                    }

                    let qos_str: Option<String> = user_input.get(1).and_then(|v| v.parse().ok());
                    let qos: u8 = qos_str
                        .clone()
                        .unwrap_or_else(|| String::from(""))
                        .parse()
                        .unwrap_or(0);

                    let topic_name_str: Option<String> =
                        user_input.get(2).and_then(|v| v.parse().ok());
                    let topic_name = topic_name_str.clone().unwrap_or_else(|| String::from(""));

                    let packet_identifier = client.get_packet_identifier();
                    let mut qos_vec: Vec<u8> = vec![];
                    let mut topic_names: Vec<String> = vec![];
                    qos_vec.push(qos);
                    if !topic_name.is_empty() {
                        topic_names.push(topic_name.clone());
                    } else {
                        println!("Non-existent topic name value");
                        continue;
                    }
                    let packet = packet.subscribe(packet_identifier, topic_names, qos_vec);
                    println!("--> Subscribe topic: {}", topic_name);
                    client.send(packet.value());
                }

                "unsubscribe" => {
                    // unsubscribe topic_name
                    // send unsubscribe
                    if !client.is_connected() {
                        println!(" <-- Not connected to server, please connect first");
                        continue;
                    }

                    let topic_name_str: Option<String> =
                        user_input.get(1).and_then(|v| v.parse().ok());
                    let topic_name = topic_name_str.clone().unwrap_or_else(|| String::from(""));

                    if topic_name.is_empty() {
                        println!("Non-existent topic name value");
                        continue;
                    }

                    let packet_identifier = client.get_packet_identifier();
                    let topic_names: Vec<String> = vec![topic_name.clone()];

                    let packet = packet.unsubscribe(packet_identifier, topic_names);
                    println!("--> Unsubscribe topic: {}", topic_name);
                    client.send(packet.value());
                }

                "test" => {
                    println!("Test connection to localhost");
                    if !client.is_connected() {
                        // TODO: agregar en el client.connect(host, port, username, password) dentro del connect que seteen esos datos sobre el struct
                        client
                            .connect(
                                "localhost".to_string(),
                                "3333".to_string(),
                                "".to_string(),
                                "".to_string(),
                                true,
                                "".to_string(),
                                "".to_string(),
                            )
                            .expect("Error connecting");
                        let client_identifier = client.get_id_client();
                        println!("--> Trying to connect with client id {}", client_identifier);
                        println!("Send connect");
                        let mut i: usize = 0;
                        let conn_retries = client.get_connect_retries();
                        loop {
                            if i >= conn_retries {
                                println!(" <-- Not connected to server");
                                break;
                            }
                            if client.is_connected() {
                                println!(
                                    "<-- Connected to server with client id {}",
                                    client_identifier
                                );
                                break;
                            }
                            i += 1;
                            thread::sleep(time::Duration::from_millis(1000));
                            println!("Waiting for connection ... retries: {}/{}", i, conn_retries);
                        }
                    } else {
                        println!(" <-- Already connected!");
                        continue;
                    }
                    if client.is_connected() {
                        // test publish
                        let packet_identifier = client.get_packet_identifier();
                        let packet = packet.publish(
                            0,
                            1,
                            0,
                            packet_identifier,
                            "hola".to_string(),
                            "hola2".to_string(),
                        );
                        println!("Sending packet: {:?}", packet.value());
                        client.send(packet.value());

                        // test subscribe
                        let packet_identifier = client.get_packet_identifier();
                        let qos_vec: Vec<u8> = vec![0];
                        let topic_name = "test1".to_string();
                        let topic_names: Vec<String> = vec![topic_name.clone()];
                        let packet = packet.subscribe(packet_identifier, topic_names, qos_vec);
                        println!("--> Subscribe topic: {}", topic_name);
                        client.send(packet.value());

                        // test unsubscribe
                        let topic_name = "test1".to_string();
                        let packet_identifier = client.get_packet_identifier();
                        let topic_names: Vec<String> = vec![topic_name.clone()];
                        let packet = packet.unsubscribe(packet_identifier, topic_names);
                        println!("--> Unsubscribe topic: {}", topic_name);
                        client.send(packet.value());
                    }
                }

                "test-connection" => {
                    if client.is_connected() {
                        println!(
                            "<-- Connected to server with client id {}",
                            client.get_id_client()
                        );
                    } else {
                        println!("<-- Not connected to server");
                        continue;
                    }
                }

                "exit" => {
                    break;
                }

                _ => {
                    println!("Message not understood: {:?}", user_input);
                    continue;
                }
            }
        } else {
            println!("Invalid command, to exit type: exit");
        }
    }
}

/// Opciones a ejecutar por el cliente.

pub fn user_input() -> Vec<String> {
    println!("Enter an option...");
    let mut a_str = String::new();

    stdin().read_line(&mut a_str).expect("Read error");
    let input_stdin: Vec<String> = a_str
        .split_whitespace()
        .map(|x| x.parse::<String>().expect("Parse error"))
        .collect::<Vec<String>>();
    input_stdin
}

pub fn parser_str(value: String) -> String {
    value.trim().parse().expect("Wrong value!")
}

#[test]
fn test_parser() {
    let vec = ["0", "1", "3"];
    assert_eq!("0", parser_str(vec[0].to_string()))
}

#[test]
fn test_user_input() {
    let input_stdin = String::from("publish 0 0 0 topic_name some_message");
    assert_eq!(
        "publish 0 0 0 topic_name some_message",
        parser_str(input_stdin.clone())
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
