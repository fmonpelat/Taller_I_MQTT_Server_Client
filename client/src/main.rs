mod client;
use crate::client::Client;


fn main() {
    let client = Client::new("localhost","3333");

    client.connect();

    println!("Client terminated.");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_client() {
        assert_eq!(1, 1)
    }
}
