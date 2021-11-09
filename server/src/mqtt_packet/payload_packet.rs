
#[derive(Debug, Default)]
pub struct Payload {
    pub client_identifier: String,
    pub will_topic: String,
    pub will_message: String,
    pub user_name: String,
    pub password: String, // what type does password need?
}

pub trait PacketPayload {
    fn value(&self) -> Vec<u8>;
}

impl PacketPayload for Payload {
    fn value(&self) -> Vec<u8> {
        let mut payload_vec: Vec<u8> = Vec::with_capacity(1024);

        let slice: &[&dyn ::std::fmt::Display] = 
            &[&self.client_identifier, &self.will_topic, &self.will_message, &self.user_name, &self.password];
        let _parts: Vec<_> = slice.iter().map(
            |x| {
                let x_ = (x.to_string().len() as u16).to_be_bytes();
                payload_vec.extend(
                    if x.to_string().len() > 0 {
                        x_.iter()
                    } else { [].iter() }
                );
                payload_vec.extend(x.to_string().as_bytes());
                x.to_string()
            }
        ).collect();

        if payload_vec.len() == 0 {
            return vec![];
        }
        return payload_vec;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_connect_value() {
        let client_identifier = String::from("testing");
        let len_stub = client_identifier.len() + 2;
        let mut stub = vec![0];
        stub.push(client_identifier.len() as u8);
        stub.extend(client_identifier.as_bytes());

        let payload = Payload {
            client_identifier: client_identifier.clone(),
            ..Payload::default()
        };
        let value: Vec<u8> = payload.value();
        // println!("payload value len: {:?} expected: {:?}", value.len(), len_stub);
        assert!(value.len() == len_stub);
        assert!( stub.eq(&value) );
    }
}