use headerPacket::{control_flags, control_type, Header};
mod variableHeaderPacket;

#[allow(dead_code)]
pub mod connect_flags {
    pub const USERNAME: u8 = 0x80;
    pub const PASSWORD: u8 = 0x40;
    pub const WILL_RETAIN: u8 = 0x20;
    pub const WILL_QOS1: u8 = 0x10;
    pub const WILL_QOS2: u8 = 0x08;
    pub const WILL: u8 = 0x04;
    pub const CLEAN_SESSION: u8 = 0x02;
    pub const RESERVED: u8 = 0x01;
}
  
#[derive(Debug, Default)]
pub struct VariableHeader {
    pub protocol_name: [u8;6], // must be allways: [0,4,'M','Q','T','T']
    pub protocol_level: u8, // for spec 3.1.1 mqtt the value of protocol is 4 (0x04)
    pub connect_flags: u8, // 1 byte - note: to set use CONNECTFLAGS enum
    pub keep_alive: u16, // 2 bytes
}

pub trait PacketVariableHeader {
    fn value(&self) -> Vec<u8>;
}

impl PacketVariableHeader for VariableHeader {
    fn value(&self) -> Vec<u8> {
        let mut variable_header_vec: Vec<u8> = Vec::with_capacity(12);
        for i in self.protocol_name {
            variable_header_vec.push(i);
        }
        variable_header_vec.push(self.protocol_level);
        variable_header_vec.push(self.connect_flags);
        variable_header_vec.push((self.keep_alive >> 8) as u8);
        variable_header_vec.push((self.keep_alive & 0xFF) as u8);
        return variable_header_vec;
    }
}

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
#[derive(Debug, Default)]
pub struct Packet {
    header: Header,
    variable_header: VariableHeader,
    payload: Payload,
}

pub trait ModPacket {
    fn new() -> Packet;
    fn connect(&self, client_identifier: String) -> Packet;
    fn connack(&self) -> Packet;
    fn value(&self) -> Vec<u8>; // this gives us the bytestream directly to send through stream
}
     
impl ModPacket for Packet {
    fn connect(&self, client_identifier: String) -> Packet {
        let header = Header {
            control_type: control_type::CONNECT, // 0x10
            control_flags: control_flags::RESERVED, // 0x00
            remaining_length_0: 0x00, // what remaining lenght is? (how it is calculated)
        };
        let variable_header = VariableHeader {
            protocol_name: [0, 4, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8],
            protocol_level: 0x04,
            connect_flags: connect_flags::CLEAN_SESSION, // what connect flags do i need?
            keep_alive: 0x00,
        };
        // do we need a payload if there is a connect?
        let payload = Payload {
            client_identifier: client_identifier,
            ..Payload::default()
        };

        // building the struct packet
        let packet = Packet {
            header: header,
            variable_header: variable_header,
            payload: payload,
        };
        return packet;
    }

    fn connack(&self) -> Packet {
        todo!()
    }

    fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        let vec: Vec<u8> = self.header.value().iter().cloned().chain(
            self.variable_header.value().iter().cloned()).chain(
                self.payload.value().iter().cloned()
        ).collect();
        for i in vec {
            res.push(i);
        }
        return res;
    }

    fn new() -> Packet {
        Packet {
            header: Header::default(),
            variable_header: VariableHeader::default(),
            payload: Payload::default(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod packets {
        use super::*;
        #[test]
        fn test_connect_packet() {
            let connect_head_stub = vec![16, 0, 0, 4, 77, 81, 84, 84, 4, 2, 0, 0];
            let client_identifier = String::from("testId");
            let connect_stub: Vec<u8> = connect_head_stub.iter().copied().chain(
                (client_identifier.len() as u16).to_be_bytes().iter().copied().chain(
                    client_identifier.as_bytes().iter().copied()
                )
            ).collect();
            let packet = Packet::new();
            let packet = packet.connect(client_identifier);
            let value = packet.value();
            // println!("value connect: {:?}", value);
            // println!("connect stub: {:?}", connect_stub);
            assert_eq!(value.len(), connect_stub.len());
            assert!(connect_stub.eq(&value));
        }
    }
}
