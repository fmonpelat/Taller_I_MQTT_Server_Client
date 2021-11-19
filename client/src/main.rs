mod client;
use std::io::stdin;
use crate::client::Client;
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet, ServerPacket};
use mqtt_packet::mqtt_packet_service::variable_header_packet::{VariableHeader};
use mqtt_packet::mqtt_packet_service::payload_packet::{Payload};


fn main() {
    let client = Client::new(String::from("localhost"),String::from("3333"));

    println!("MQTT Client V1.0\n");
    client.connect();
    
    client.publish(String::from("test"),String::from("test"));

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
                client.connect();
                let packet = packet.connect(String::from(&user_input[1]));
                client.send(packet.value());
                println!("send connect");
            },
            "publish" => {
                // send publish
                //let packet = packet.publish();
                if client.is_connect() {
                    client.send(packet.value());
                    println!("send publish");
                }
            },
            "pingreq" => {
                // send publish
                if client.is_connect(){
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
