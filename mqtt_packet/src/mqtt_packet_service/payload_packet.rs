/// A Payload for a mqtt packet
/// There are several payloads for a mqtt packet
///
/// Connect has the payload type Payload
#[derive(Debug, Default)]
pub struct Payload {
    pub client_identifier: String,
    pub will_topic: String,
    pub will_message: String,
    pub user_name: String,
    pub password: String,
}

/// Publish has the payload type PublishPayload
#[derive(Debug, Default)]
pub struct PublishPayload {
    pub message: String,
}

pub trait PacketPayload {
    fn value(&self) -> Vec<u8>;
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> Payload;
}

impl PacketPayload for Payload {
    /// Unvalue deserializes the vector of bytes to a Payload
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> Payload {
        *readed = 0;
        let mut index = x.len();
        if index == 0 {
            return Payload::default();
        }
        let mut client_identifier_len: u16 = 0;
        let mut will_topic_len: u16 = 0;
        let mut will_message_len: u16 = 0;
        let mut user_name_len: u16 = 0;
        let mut password_len: u16 = 0;
        let mut client_identifier = String::new();
        let mut will_topic = String::new();
        let mut will_message = String::new();
        let mut user_name = String::new();
        let mut password = String::new();

        if index > 0 {
            client_identifier_len = x[0] as u16 + (x[1] as u16);
            index = index - 2;
            client_identifier =
                String::from_utf8(x[2..(2 + client_identifier_len as usize)].to_vec()).unwrap();
            // println!("client_identifier_len: {}", client_identifier_len);
            // println!("client_identifier: {}", client_identifier);
            index = index - client_identifier_len as usize;
        }

        if index > 0 {
            will_topic_len = x[2 + client_identifier_len as usize] as u16
                + (x[3 + client_identifier_len as usize] as u16);
            index = index - 2;
            will_topic = String::from_utf8(
                x[4 + client_identifier_len as usize
                    ..(4 + client_identifier_len as usize + will_topic_len as usize)]
                    .to_vec(),
            )
            .unwrap();
            index = index - will_topic_len as usize;
            // println!("will_topic_len: {}", will_topic_len);
            // println!("will_topic: {}", will_topic);
        }

        if index > 0 {
            will_message_len = x[4 + client_identifier_len as usize + will_topic_len as usize]
                as u16
                + (x[5 + client_identifier_len as usize + will_topic_len as usize] as u16);
            index = index - 2;
            will_message = String::from_utf8(
                x[6 + client_identifier_len as usize + will_topic_len as usize
                    ..(6 + client_identifier_len as usize
                        + will_topic_len as usize
                        + will_message_len as usize)]
                    .to_vec(),
            )
            .unwrap();
            index = index - will_message_len as usize;
            // println!("will_message_len: {}", will_message_len);
            // println!("will_message: {}", will_message);
        }

        if index > 0 {
            user_name_len = x[6
                + client_identifier_len as usize
                + will_topic_len as usize
                + will_message_len as usize] as u16
                + (x[7
                    + client_identifier_len as usize
                    + will_topic_len as usize
                    + will_message_len as usize] as u16);
            index = index - 2;
            user_name = String::from_utf8(
                x[8 + client_identifier_len as usize
                    + will_topic_len as usize
                    + will_message_len as usize
                    ..(8 + client_identifier_len as usize
                        + will_topic_len as usize
                        + will_message_len as usize
                        + user_name_len as usize)]
                    .to_vec(),
            )
            .unwrap();
            index = index - user_name_len as usize;
            // println!("user_name_len: {}", user_name_len);
            // println!("user_name: {}", user_name);
        }

        if index > 0 {
            password_len = x[8
                + client_identifier_len as usize
                + will_topic_len as usize
                + will_message_len as usize
                + user_name_len as usize] as u16
                + (x[9
                    + client_identifier_len as usize
                    + will_topic_len as usize
                    + will_message_len as usize
                    + user_name_len as usize] as u16);
            index = index - 2;
            password = String::from_utf8(
                x[10 + client_identifier_len as usize
                    + will_topic_len as usize
                    + will_message_len as usize
                    + user_name_len as usize
                    ..(10
                        + client_identifier_len as usize
                        + will_topic_len as usize
                        + will_message_len as usize
                        + user_name_len as usize
                        + password_len as usize)]
                    .to_vec(),
            )
            .unwrap();
            index = index - password_len as usize;
            // println!("password_len: {}", password_len);
            // println!("password: {}", password);
        }
        *readed = x.len() - index;

        Payload {
            client_identifier: client_identifier,
            will_topic: will_topic,
            will_message: will_message,
            user_name: user_name,
            password: password,
        }
    }

    /// Value serializes the payload to a vector of bytes
    /// # Examples
    /// Basic Usage:
    /// ```
    /// let payload = Payload { ..Payload::default() };
    /// let value: Vec<u8> = payload.value();
    /// ```
    fn value(&self) -> Vec<u8> {
        let mut payload_vec: Vec<u8> = Vec::with_capacity(1024);

        let slice: &[&dyn ::std::fmt::Display] = &[
            &self.client_identifier,
            &self.will_topic,
            &self.will_message,
            &self.user_name,
            &self.password,
        ];
        let _parts: Vec<_> = slice
            .iter()
            .map(|x| {
                let x_ = (x.to_string().len() as u16).to_be_bytes();
                payload_vec.extend(if !x.to_string().is_empty() {
                    x_.iter()
                } else {
                    [].iter()
                });
                payload_vec.extend(x.to_string().as_bytes());
                x.to_string()
            })
            .collect();

        if payload_vec.is_empty() {
            return vec![];
        }
        payload_vec
    }
}

pub trait PacketPublishPayload {
    fn value(&self) -> Vec<u8>;
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> PublishPayload;
}

impl PacketPublishPayload for PublishPayload {
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> PublishPayload {
        *readed = 0;
        if x.len() == 0 {
            return PublishPayload::default();
        }
        let payload_len = x[0] as u16 + (x[1] as u16);
        let payload = String::from_utf8(x[2..(2 + payload_len as usize)].to_vec()).unwrap();
        *readed = 2 + payload_len as usize;
        PublishPayload { message: payload }
    }
    fn value(&self) -> Vec<u8> {
        let mut payload_vec: Vec<u8> = Vec::with_capacity(1024);
        self.message
            .as_bytes()
            .iter()
            .for_each(|x| payload_vec.push(*x));
        if payload_vec.is_empty() {
            return vec![];
        }
        payload_vec
    }
}

#[cfg(test)]
mod tests {
    use std::ptr::read_unaligned;

    use super::*;

    #[test]
    fn payload_unpopulated_unvalue() {
        let payload = Payload {
            ..Payload::default()
        };
        let value: Vec<u8> = payload.value();
        let mut readed = 0;
        let unvalue = Payload::unvalue(value.clone(), &mut readed);
        assert_eq!(payload.client_identifier, unvalue.client_identifier);
        assert_eq!(payload.will_topic, unvalue.will_topic);
        assert_eq!(payload.will_message, unvalue.will_message);
        assert_eq!(payload.user_name, unvalue.user_name);
        assert_eq!(payload.password, unvalue.password);
        assert_eq!(readed, value.len());
    }

    #[test]
    fn payload_populated_unvalue() {
        let client_identifier = "test";
        let payload = Payload {
            client_identifier: client_identifier.to_string(),
            will_topic: "will_test".to_string(),
            will_message: "will_message".to_string(),
            user_name: "user_name_test".to_string(),
            password: "password_test".to_string(),
        };
        let value: Vec<u8> = payload.value();
        let mut readed = 0;
        let unvalue = Payload::unvalue(value.clone(), &mut readed);
        assert_eq!(payload.client_identifier, unvalue.client_identifier);
        assert_eq!(payload.will_topic, unvalue.will_topic);
        assert_eq!(payload.will_message, unvalue.will_message);
        assert_eq!(payload.user_name, unvalue.user_name);
        assert_eq!(payload.password, unvalue.password);
        assert_eq!(readed, value.len());
    }

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
        // let mut readed: usize = 0;
        // let unvalue = Payload::unvalue(value.clone(), &mut readed);
        // println!("{:?}",unvalue);
        assert!(value.len() == len_stub);
        assert!(stub.eq(&value));
    }
}
