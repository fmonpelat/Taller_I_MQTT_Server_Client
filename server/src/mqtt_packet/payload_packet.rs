
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

        let client_identifier_ = (self.client_identifier.len() as u16).to_be_bytes();
        payload_vec.extend(
            if self.client_identifier.len() > 0 {
                client_identifier_.iter()
            } else { [].iter() }
        );
        payload_vec.extend(self.client_identifier.as_bytes());


        let will_topic_ = (self.will_topic.len() as u16).to_be_bytes();
        payload_vec.extend(
            if self.will_topic.len() > 0 {
                will_topic_.iter()
            } else { [].iter() }
        );
        payload_vec.extend(self.will_topic.as_bytes());


        let will_message_ = (self.will_message.len() as u16).to_be_bytes();
        payload_vec.extend(
            if self.will_message.len() > 0 {
                will_message_.iter()
            } else { [].iter() }
        );
        payload_vec.extend(self.will_message.as_bytes());


        let user_name_ = (self.user_name.len() as u16).to_be_bytes();
        payload_vec.extend(
            if self.user_name.len() > 0 {
                user_name_.iter()
            } else { [].iter() }
        );
        payload_vec.extend(self.user_name.as_bytes());


        let password_ = (self.password.len() as u16).to_be_bytes();
        payload_vec.extend(
                if self.password.len() > 0 {
                    password_.iter()
                } else { [].iter() }
        );
        payload_vec.extend(self.password.as_bytes());


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
        let client_identifier = String::from("test");
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