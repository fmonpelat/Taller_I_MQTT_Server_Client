use client::client::Client;
use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::VariableHeader;
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet};
use rand::distributions::Uniform;
use rand::Rng;
use std::thread::sleep;
use std::time::Duration;
mod file_loader;
use crate::file_loader::load_contents;
use core::time;
use std::thread;

fn main() {
    let file_config = "src/config.yaml";
    let config = load_contents(file_config);
    let host = config
        .get("host")
        .unwrap_or_else(|| panic!("Cannot found host in config"));
    let port = config
        .get("port")
        .unwrap_or_else(|| panic!("Cannot found port in config"));
    let topic = config
        .get("topic")
        .unwrap_or_else(|| panic!("Cannot found topic in config"));
    let default_user = String::from("");
    let user: String = config
        .get("user")
        .unwrap_or_else(|| &default_user)
        .parse()
        .unwrap();
    let default_password = String::from("");
    let password: String = config
        .get("password")
        .unwrap_or_else(|| &default_password)
        .parse()
        .unwrap();
    let default_interval_time = String::from("15");
    let interval_time: usize = config
        .get("interval_time")
        .unwrap_or_else(|| &default_interval_time)
        .parse()
        .unwrap();
    let default_min_interval_num = String::from("1111");
    let min_interval_num: i32 = config
        .get("min_interval_num")
        .unwrap_or_else(|| &default_min_interval_num)
        .parse()
        .unwrap();
    let default_max_interval_num = String::from("9999");
    let max_interval_num: i32 = config
        .get("max_interval_num")
        .unwrap_or_else(|| &default_max_interval_num)
        .parse()
        .unwrap();

    handle_connection(
        host.to_string(),
        port.to_string(),
        topic.to_string(),
        user.to_string(),
        password.to_string(),
        interval_time,
        min_interval_num,
        max_interval_num,
    )
}

fn handle_connection(
    host: String,
    port: String,
    topic: String,
    user: String,
    password: String,
    interval_time: usize,
    min_interval_num: i32,
    max_interval_num: i32,
) -> () {
    let mut client = Client::new();
    println!("Connection to the server");
    client
        .connect(
            host.to_owned(),
            port.to_owned(),
            user.to_owned(),
            password.to_owned(),
            true,
            "".to_string(),
            "".to_string(),
        )
        .expect("Error connecting");

    let client_identifier = client.get_id_client();
    println!("--> Trying to connect with client id {}", client_identifier);
    let mut i: usize = 0;
    let conn_retries = client.get_connect_retries();
    loop {
        if i >= conn_retries {
            // print something as error on loop
            client.disconnect();
            panic!("Connection to client failed.");
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
    
    loop {
        // Creating a new default mqtt packet
        let packet: Packet<VariableHeader, Payload> = Packet::<VariableHeader, Payload>::new();

        if client.is_connected() {
            // Publish to the server
            println!("Publishing the topic to the server");
            let packet_identifier = client.get_packet_identifier();
            let mut rng = rand::thread_rng();
            let interval = Uniform::from(min_interval_num..max_interval_num);
            let token_rng: i32 = rng.sample(interval);
            let packet = packet.publish(
                0,
                1,
                0,
                packet_identifier,
                topic.to_string(),
                token_rng.to_string(),
            );
            println!("Sending packet: {:?}", packet.value());
            client.send(packet.value())
        } else {
            // Send a panic message when the client is disconnected
            panic!("Client disconnected ");
        }
        println!("Client will publish soon");
        sleep(Duration::new(interval_time as u64, 0));
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
