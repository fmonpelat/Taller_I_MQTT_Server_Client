mod client;
use std::io::stdin;

use crate::client::Client;


fn main() {
    let client = Client::new(String::from("localhost"),String::from("3333"));

    println!("MQTT Client V1.0\n");
    client.connect();

    client.publish(String::from("test"),String::from("test"));

    loop {
        // read from stdin and send to server
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        match input.as_ref() {
            "exit\n" => {
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
