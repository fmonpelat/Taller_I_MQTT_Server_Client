#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_value() {
        let client_identifier = String::from("test");
        let payload = Payload {
            client_identifier: client_identifier.clone(),
            ..Payload::default()
        };
        let value: Vec<u8> = payload.value();
        // println!("payload value: {:?}", value);
        assert!(value.len() == client_identifier.len());
        assert!( client_identifier.as_bytes().eq(&value) );
    }
}