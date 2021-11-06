#[allow(dead_code)]
pub mod control_type {
    pub const CONNECT: u8 = 0x10;
    pub const CONNACK: u8 = 0x20;
    pub const PUBLISH: u8 = 0x30;
    pub const PUBACK: u8 = 0x40;
    pub const PUBREC: u8 = 0x50;
    pub const PUBREL: u8 = 0x60;
    pub const PUBCOMP: u8 = 0x70;
    pub const SUBSCRIBE: u8 = 0x80;
    pub const SUBACK: u8 = 0x90;
    pub const UNSUBSCRIBE: u8 = 0xA0;
    pub const UNSUBACK: u8 = 0xB0;
    pub const PINGREQ: u8 = 0xC0;
    pub const PINGRESP: u8 = 0xD0;
    pub const DISCONNECT: u8 = 0xE0;
    pub const RESERVED: u8 = 0xF0;
}

#[allow(dead_code)]
pub mod control_flags {
    pub const RESERVED: u8 = 0x00;
    pub const RETAIN: u8 = 0x01;
    pub const QOS0: u8 = 0x02;
    pub const QOS1: u8 = 0x04;
    pub const DUP: u8 = 0x08;
}

#[allow(dead_code)]
pub mod connect_flags {
    pub const USERNAME: u8 = 0x7;
    pub const PASSWORD: u8 = 0x6;
    pub const WILL_RETAIN: u8 = 0x5;
    pub const WILL_QOS1: u8 = 0x4;
    pub const WILL_QOS2: u8 = 0x3;
    pub const WILL: u8 = 0x2;
    pub const CLEAN_SESSION: u8 = 0x1;
    pub const RESERVED: u8 = 0x0;
}


#[derive(Debug, Default)]
pub struct Header {
    // total length of 4 bytes (16 bits)
    pub control_type: u8, // msb byte
    pub control_flags: u8, // lsb byte
    pub remaining_length_0: u8, // 1 byte
}
// traits and impl for header
pub trait PacketHeader {
    fn get_cmd_type(&self) -> u8;
    fn get_cmd_flags(&self) -> u8;
    fn decode_remaining_length(&self) -> u32;
    fn encode_remaining_header(&self, x: u32) -> u32;
    fn value(&self) -> Vec<u8>;
}

impl PacketHeader for Header {

    fn value(&self) -> Vec<u8> {
        let mut header_vec: Vec<u8> = Vec::with_capacity(3);
        header_vec.push(self.get_cmd_type() + self.get_cmd_flags());
        header_vec.push(self.remaining_length_0);
        return header_vec;
    }

    fn get_cmd_type(&self) -> u8 {
        self.control_type
    }

    fn get_cmd_flags(&self) -> u8 {
        self.control_flags
    }

    fn decode_remaining_length(&self) -> u32 {
        let mut multiplier = 1;
        let mut value = 0;
        while (self.remaining_length_0 & 128) != 0 {
            value += (self.remaining_length_0 & 127) * multiplier;
            multiplier *= 128;
            if multiplier > 128*128*128 {
                panic!("Malformed Remaining Length")
            }
        }
        return value as u32;
    }

    fn encode_remaining_header(&self, x: u32) -> u32 {
        let mut digit = 0;
        let mut x_ = x;
        while x_ > 0 {
            digit = x % 128;
            x_ = x_ / 128;
            if x_ > 0 {
                digit |= 128;
            }
        }
        return digit;
    }
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
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        res.extend(self.client_identifier.as_bytes());
        res.extend(self.will_topic.as_bytes());
        res.extend(self.will_message.as_bytes());
        res.extend(self.user_name.as_bytes());
        res.extend(self.password.as_bytes());
        if res.len() == 0 {
            return vec![];
        }
        return res;
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
    
    mod values {
        use super::*;
        #[test]
        fn header_connect_value() {
            let control_type = control_type::CONNECT;
            let control_flags = control_flags::RESERVED;
            let remaining_length_0 = 0x00;
            let header_stub = vec![control_type + control_flags, remaining_length_0];
            let header = Header {
                control_type: control_type,
                control_flags: control_flags,
                remaining_length_0: remaining_length_0,
            };
            
            let value: Vec<u8> = header.value();
            assert!(value.len() == header_stub.len());
            assert!(header_stub.eq(&value));
        }
        #[test]
        fn variable_header_value() {
            let protocol_name = [0, 4, 'M' as u8, 'Q' as u8, 'T' as u8, 'T' as u8];
            let protocol_level = 0x04;
            let connect_flags = connect_flags::CLEAN_SESSION;
            let keep_alive = 0x00;
            let vh_stub: Vec<u8> = protocol_name.iter().copied().chain(
                vec![protocol_level, connect_flags, (keep_alive >> 8) as u8, (keep_alive & 0xFF) as u8]
            ).collect();
            
            let variable_header = VariableHeader {
                protocol_name: protocol_name,
                protocol_level: protocol_level,
                connect_flags: connect_flags,
                keep_alive: keep_alive,
            };
            let value: Vec<u8> = variable_header.value();
            println!("variable header value: {:?}", value);
            assert!(value.len() == vh_stub.len());
            assert!(vh_stub.eq(&value));
        }

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

    mod packets {
        use super::*;
        #[test]
        fn test_connect_packet() {
            let connect_head_stub = vec![16, 0, 0, 4, 77, 81, 84, 84, 4, 1, 0, 0];
            let client_identifier = String::from("testId");
            let connect_stub: Vec<u8> = connect_head_stub.iter().copied().chain(
                client_identifier.as_bytes().iter().copied()
            ).collect();
            let packet = Packet::new();
            let packet = packet.connect(client_identifier);
            let value = packet.value();
            // println!("value connect: {:?}", value);

            assert_eq!(value.len(), connect_stub.len());
            assert!(connect_stub.eq(&value));
        }
    }
}
