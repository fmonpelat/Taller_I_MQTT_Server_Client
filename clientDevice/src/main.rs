mod clientDevice;
use crate::clientDevice::ClientDevice;
use core::time;
use mqtt_packet::mqtt_packet_service::payload_packet::Payload;
use mqtt_packet::mqtt_packet_service::variable_header_packet::VariableHeader;
use mqtt_packet::mqtt_packet_service::{ClientPacket, Packet};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;

fn main() {
    let mut client = ClientDevice::new();

    println!("MQTT Client V2.0\n");

    loop {

        // creating a new default mqtt packet
        let packet: Packet<VariableHeader, Payload> = Packet::<VariableHeader, Payload>::new();
        println!("######## Connection to localhost ########");
        client
            .connect(
                "localhost".to_string(),
                "3333".to_string(),
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
        if !client.is_connected() {
            println!(
                "Client is not connected to server. Retrying ",
            );
            continue;
        }

        if client.is_connected() {
            // Publish to the server
            println!("######## Publishing the topic to the server ########");
            let packet_identifier = client.get_packet_identifier();
            let mut rng = rand::thread_rng();
            let temperature = rng.gen_range(-100..100);
            let packet = packet.publish(
                0,
                1,
                0,
                packet_identifier,
                "temperature".to_string(),
                temperature.to_string(),
            );
            println!("Sending packet: {:?}", packet.value());
            client.send(packet.value());

            // Disconnect from the server   
            client.disconnect();
            println!("--> Disconnected from server.");

        }
        println!("########## Client will publish in 30 seconds ########");
        sleep(Duration::new(30, 0));       
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
