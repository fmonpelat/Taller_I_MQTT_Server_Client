mod client;
use std::io::stdin;
use crate::client::{Client,ClientConected};
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet, ServerPacket};
use mqtt_packet::mqtt_packet_service::variable_header_packet::{VariableHeader};
use mqtt_packet::mqtt_packet_service::payload_packet::{Payload};
use std::sync::atomic::{Ordering};


fn main() {
    let client = Client::new(String::from("localhost"),String::from("3333"));

    println!("MQTT Client V1.0\n");
    //client.connect();
    let mut isConnected = ClientConected::new();
    loop {
        // read from stdin and send to server
        //let mut input = String::new();
        

        // si el usuario se conecta -->connect
        // si quiere  publish  --> checquear si esta connectado
        // luego hace match

        let packet: Packet::<VariableHeader, Payload> = Packet::<VariableHeader, Payload>::new();         
        let user_input = user_input();
        
        match user_input[0].to_lowercase().as_ref() {
            "connect" => {
                isConnected = client.connect();
                let packet = packet.connect(client.get_id_client());
                client.send(packet.value());
                println!("send connect");
            },
            "publish" => {
                // send publish
                
                let dup_str: Option<String> = user_input.get(1).and_then(|v| {v.parse().ok()});
                let qos_str: Option<String> = user_input.get(2).and_then(|v| {v.parse().ok()});
                let retain_str: Option<String> = user_input.get(3).and_then(|v| {v.parse().ok()});
                let topic_name_str: Option<String> = user_input.get(4).and_then(|v| {v.parse().ok()});
                let message_str: Option<String> = user_input.get(5).and_then(|v| {v.parse().ok()});
                let mut dup: u8=0;
                let mut qos: u8=0;
                let mut retain: u8=0; 
                let mut topic_name: String =String::from("");
                let mut message: String=String::from(""); 
                
                match dup_str {
                    Some(_) => {dup = user_input[1].trim().parse()
                    .expect("wrong value!");
                },
                     None => println!("non-existent dup value"),
                }
                match qos_str {
                    Some(_) => {qos = user_input[2].trim().parse()
                    .expect("wrong value!");
                },
                     None => println!("non-existent qos value"),
                }
                match retain_str {
                    Some(_) => {retain = user_input[3].trim().parse()
                    .expect("wrong value!");
                },
                     None => println!("non-existent retain value"),
                }
                match topic_name_str {
                    Some(_) => {topic_name = user_input[4].clone();
                },
                     None => println!("non-existent topic name value"),
                }
                match message_str {
                    Some(_) => {message = user_input[5].clone();
                },
                     None => println!("non-existent message value"),
                }

                let packet = packet.publish(dup,qos,retain,topic_name,message);
                if isConnected.isConnected.load(Ordering::SeqCst) {
                    client.send(packet.value());
                    println!("send publish");
                }
            },
            "pingreq" => {
                // send publish
                if isConnected.isConnected.load(Ordering::SeqCst) {
                    let packet = packet.pingresp();
                
                    client.send(packet.value());
                    println!("send pingreq");
                }
            },
            "disconnect" =>{ //client.disconnect();
                break;
            },
            _ => {
                println!("Message not understood: {:?}",user_input);
                // print to stdout not understand
            }
        }

    };
    
    // println!("Client terminated.");
}

/// Opciones a ejecutar por el cliente.

pub fn user_input() -> Vec<String>{
    println!("Enter option...");
    let mut a_str = String::new();

    stdin().read_line(&mut a_str).expect("read error");
    let input_stdin:Vec<String> = a_str.split_whitespace().map(|x| x.parse::<String>().expect("parse error"))
    .collect::<Vec<String>>();
    
    println!("{:?}",input_stdin);
    input_stdin
}


#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
