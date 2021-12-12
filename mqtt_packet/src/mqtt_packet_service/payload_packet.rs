/// A Payload for a mqtt packet
/// There are several payloads for a mqtt packet
///

/// The return code for a suback packet
#[allow(dead_code)]
pub mod suback_return_codes {
    pub const SUCCESS_QOS0: u8 = 0x00;
    pub const SUCCESS_QOS1: u8 = 0x01;
    pub const SUCCESS_QOS2: u8 = 0x02;
    pub const FAILURE: u8 = 0x80;
}

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
        let mut client_identifier = String::new();
        let mut will_topic = String::new();
        let mut will_message = String::new();
        let mut user_name = String::new();
        let mut password = String::new();

        if index > 0 {
            client_identifier_len = x[0] as u16 + (x[1] as u16);
            index -= 2;
            client_identifier =
                String::from_utf8(x[2..(2 + client_identifier_len as usize)].to_vec()).unwrap();
            // println!("client_identifier_len: {}", client_identifier_len);
            // println!("client_identifier: {}", client_identifier);
            index -= client_identifier_len as usize;
        }

        if index > 0 {
            will_topic_len = x[2 + client_identifier_len as usize] as u16
                + (x[3 + client_identifier_len as usize] as u16);
            index -= 2;
            will_topic = String::from_utf8(
                x[4 + client_identifier_len as usize
                    ..(4 + client_identifier_len as usize + will_topic_len as usize)]
                    .to_vec(),
            )
            .unwrap();
            index -= will_topic_len as usize;
            // println!("will_topic_len: {}", will_topic_len);
            // println!("will_topic: {}", will_topic);
        }

        if index > 0 {
            will_message_len = x[4 + client_identifier_len as usize + will_topic_len as usize]
                as u16
                + (x[5 + client_identifier_len as usize + will_topic_len as usize] as u16);
            index -= 2;
            will_message = String::from_utf8(
                x[6 + client_identifier_len as usize + will_topic_len as usize
                    ..(6 + client_identifier_len as usize
                        + will_topic_len as usize
                        + will_message_len as usize)]
                    .to_vec(),
            )
            .unwrap();
            index -= will_message_len as usize;
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
            index -= 2;
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
            index -= user_name_len as usize;
            // println!("user_name_len: {}", user_name_len);
            // println!("user_name: {}", user_name);
        }

        if index > 0 {
            let password_len = x[8
                + client_identifier_len as usize
                + will_topic_len as usize
                + will_message_len as usize
                + user_name_len as usize] as u16
                + (x[9
                    + client_identifier_len as usize
                    + will_topic_len as usize
                    + will_message_len as usize
                    + user_name_len as usize] as u16);
            index -= 2;
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
            index -= password_len as usize;
            // println!("password_len: {}", password_len);
            // println!("password: {}", password);
        }
        *readed = x.len() - index;

        Payload {
            client_identifier,
            will_topic,
            will_message,
            user_name,
            password,
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
                payload_vec.extend(x_.iter());
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
        if x.is_empty() {
            return PublishPayload::default();
        }
        let payload_len = x[0] as u16 + (x[1] as u16);
        let payload = String::from_utf8(x[2..(2 + payload_len as usize)].to_vec())
            .unwrap_or_else(|_| String::from(""));
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

/// Suscribe has the payload type SuscribePayload
#[derive(Debug, Default)]
pub struct SuscribePayload {
    pub topic_filter: Vec<String>,
    pub qos: Vec<u8>,
}
pub trait PacketPayloadSuscribe {
    fn value(&self) -> Vec<u8>;
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> SuscribePayload;
}

impl PacketPayloadSuscribe for SuscribePayload {
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> SuscribePayload {
        *readed = 0;
        if x.is_empty() {
            return SuscribePayload::default();
        }
        let mut index = 0; // index of the payload value
        let mut topic_filter = Vec::new();
        let mut qos = Vec::new();

        while index < x.len() {
            let topic_filter_len = x[index] as u16 + (x[index + 1] as u16);
            index += 2;
            let topic_filter_ =
                String::from_utf8(x[index..(index + topic_filter_len as usize)].to_vec())
                    .unwrap_or_else(|_| String::from("")); // topic_filter default empty as error

            index += topic_filter_len as usize; // index of the next topic_filter length
            topic_filter.push(topic_filter_);
            let qos_ = x[index];
            index += 1;
            qos.push(qos_);
        }
        *readed = index;
        SuscribePayload { topic_filter, qos }
    }

    fn value(&self) -> Vec<u8> {
        let mut payload_vec: Vec<u8> = Vec::with_capacity(2048); // 2KB max payload
        let mut i: usize = 0;
        self.topic_filter.iter().for_each(|x| {
            let x_ = (x.len() as u16).to_be_bytes();
            payload_vec.extend(x_); // extend payload_vec with topic_filter[i] length
            payload_vec.extend(x.as_bytes()); // extend payload_vec with topic_filter[i]
            payload_vec.push(self.qos[i] & 0x03); // extend payload_vec with qos[i] (ensuring that we use only the first 2 bits)
            i += 1;
        });
        if payload_vec.is_empty() {
            return vec![];
        }
        payload_vec
    }
}

/// Suscribe has the payload type SuscribePayload
#[derive(Debug, Default)]
pub struct SubackPayload {
    pub qos: Vec<u8>,
}
pub trait PacketSubackPayload {
    fn value(&self) -> Vec<u8>;
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> SubackPayload;
}

impl PacketSubackPayload for SubackPayload {
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> SubackPayload {
        *readed = 0;
        if x.is_empty() {
            return SubackPayload::default();
        }
        *readed = x.len();
        SubackPayload { qos: x }
    }

    fn value(&self) -> Vec<u8> {
        let mut payload_vec: Vec<u8> = Vec::with_capacity(2048); // 2KB max payload
        self.qos.iter().for_each(|x| {
            payload_vec.push(*x);
        });
        if payload_vec.is_empty() {
            return vec![];
        }
        payload_vec
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn suback_payload_test() {
        let mut readed = 0;
        let payload = SubackPayload {
            qos: vec![
                suback_return_codes::SUCCESS_QOS0,
                suback_return_codes::SUCCESS_QOS1,
                suback_return_codes::FAILURE,
            ],
        };
        let payload_vec = payload.value();
        // println!("payload_vec: {:?}", payload_vec);
        let payload_ = SubackPayload::unvalue(payload_vec.clone(), &mut readed);
        assert_eq!(payload.qos, payload_.qos);
        assert_eq!(readed, payload_vec.len());
    }

    #[test]
    fn suscribe_payload_test() {
        let topic1 = "topic1";
        let topic2 = "topic2";
        let payload = SuscribePayload {
            topic_filter: vec![String::from(topic1), String::from(topic2)],
            qos: vec![0, 1],
        };
        let value = payload.value();
        assert_eq!(value.len(), topic1.len() + topic2.len() + 4 + 2); // 4 bytes for topic_filter length and 2 bytes for qos
        assert_eq!((value[0] + value[1]) as u8, topic1.len() as u8);
        assert_eq!(
            String::from_utf8(value[2..(2 + topic1.len() as usize)].to_vec())
                .unwrap_or_else(|_| String::from("")),
            String::from(topic1)
        );
        assert_eq!(value[2 + topic1.len() as usize], 0); // qos topic1 is 0
        let topic2_len =
            (value[3 + topic1.len() as usize] + value[4 + topic1.len() as usize]) as u8;
        assert_eq!(topic2_len as u8, topic2.len() as u8);
        assert_eq!(
            String::from_utf8(
                value[5 + topic1.len() as usize..(5 + topic1.len() as usize + topic2_len as usize)]
                    .to_vec()
            )
            .unwrap_or_else(|_| String::from("")),
            String::from(topic2)
        );
        assert_eq!(value[5 + topic1.len() as usize + topic2_len as usize], 1); // qos topic2 is 1
        let readed = &mut 0;
        let payload_ = SuscribePayload::unvalue(value, readed);
        assert!(payload.topic_filter == payload_.topic_filter);
        assert!(payload.qos == payload_.qos);
    }

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

        let payload = Payload {
            client_identifier: client_identifier.to_string(),
            will_topic: "".to_string(),
            will_message: "".to_string(),
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
