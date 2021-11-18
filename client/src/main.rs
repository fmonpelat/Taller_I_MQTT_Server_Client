mod client;
use std::io::stdin;
use crate::client::Client;
use mqtt_packet::mqtt_packet_service::{ClientPacket,Packet};
use mqtt_packet::mqtt_packet_service::variable_header_packet::{VariableHeader};
use mqtt_packet::mqtt_packet_service::payload_packet::{Payload};


fn main() {
    let client = Client::new(String::from("localhost"),String::from("3333"));

    println!("MQTT Client V1.0\n");
    client.connect();
    
    client.publish(String::from("test"),String::from("test"));

    let packet_client: Packet<VariableHeader, Payload>  = ClientPacket::connect(String::from("id_client"));

    show_options();
    loop {
        // read from stdin and send to server
        //let mut input = String::new();
        let input: String = input_option().to_string();
        match input.as_ref() {
            "1" => {client.connect_m(packet_client)},
            "2" => {client.publish(String::from("topic"),String::from("payload"))},
            
            "4" => {
                // client.disconnect();
                break;
            },
            _ => {
                println!("Message not understood: {:?}",input);
                // print to stdout not understand
            }
        }

    };
    
    // println!("Client terminated.");
}

/// Muestra las opciones a ejecutar por el cliente.
fn show_options() {
    println!("\t Options:");
    println!("\t\t 1. ---> Connect.");
    println!("\t\t 2. ---> Publish.");
    println!("\t\t 3. ---> Suscriber.");
    println!("\t\t 4. ---> Exit.");
}

pub fn input_option()-> String {
    println!("Enter an option...");
    let mut input = String::new();
    stdin().read_line(&mut input).ok().expect("Failed to read line");
    input
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
