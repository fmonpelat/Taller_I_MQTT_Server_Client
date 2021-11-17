
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

#[allow(dead_code)]
pub mod connect_return {
    pub const ACCEPTED: u8 = 0x00;
    pub const UNACCEPTABLE_PROTOCOL_VERSION: u8 = 0x01;
    pub const IDENTIFIER_REJECTED: u8 = 0x02;
    pub const SERVER_UNAVAILABLE: u8 = 0x03;
    pub const BAD_USERNAME_OR_PASSWORD: u8 = 0x04;
    pub const NOT_AUTHORIZED: u8 = 0x05; 
}

#[allow(dead_code)]
pub mod connect_ack_flags {
    pub const SESSION_PRESENT: u8 = 0x01;
}

#[derive(Debug, Default)]
pub struct VariableHeader {
    pub protocol_name: Vec<u8>, // must be allways: [0,4,'M','Q','T','T']
    pub protocol_level: u8, // for spec 3.1.1 mqtt the value of protocol is 4 (0x04)
    pub connect_flags: u8, // 1 byte - note: to set use CONNECTFLAGS enum
    pub keep_alive: u16, // 2 bytes
}

pub trait PacketVariableHeader {
    fn value(&self) -> Vec<u8>;
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> VariableHeader;
}

impl PacketVariableHeader for VariableHeader {
    fn value(&self) -> Vec<u8> {
        let mut variable_header_vec: Vec<u8> = Vec::with_capacity(12);
        for i in &self.protocol_name {
            variable_header_vec.push(*i);
        }
        variable_header_vec.push(self.protocol_level);
        variable_header_vec.push(self.connect_flags);
        variable_header_vec.push((self.keep_alive >> 8) as u8);
        variable_header_vec.push((self.keep_alive & 0xFF) as u8);
        variable_header_vec
    }

    fn unvalue(x: Vec<u8>, readed: &mut usize) -> VariableHeader {
        let protocol_name = x[0..6].to_vec();
        let protocol_level = x[6];
        let connect_flags = x[7];
        let keep_alive = (x[8] as u16) << 8 | (x[9] as u16);
        *readed = 10;
        VariableHeader {
            protocol_name,
            protocol_level,
            connect_flags,
            keep_alive,
        }
    }
}

#[derive(Debug, Default)]
pub struct VariableHeaderConnack {
    pub acknoledge_flags: u8, // 1 byte
    pub return_code: u8, // 1 byte
}

pub trait PacketVariableHeaderConnack {
    fn value(&self) -> Vec<u8>;
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> VariableHeaderConnack;
}

impl PacketVariableHeaderConnack for VariableHeaderConnack {
    fn value(&self) -> Vec<u8> {
        vec![self.acknoledge_flags, self.return_code]
    }
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> VariableHeaderConnack {
        *readed = 2;
        VariableHeaderConnack {
            acknoledge_flags: x[0],
            return_code: x[1],
        }
    }
}

#[derive(Debug, Default)]
pub struct VariableHeaderPublish {
    pub topic_name: Vec<u8>,
    pub packet_identifier: u16, // 2 bytes
}
pub trait PacketVariableHeaderPublish {
    fn value(&self) -> Vec<u8>;
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> VariableHeaderPublish;
}

impl PacketVariableHeaderPublish for VariableHeaderPublish {
    fn value(&self) -> Vec<u8> {
        let mut variable_header_vec: Vec<u8> = Vec::with_capacity(1024);
        let topic_name_len = self.topic_name.len();
        variable_header_vec.push((topic_name_len >> 8) as u8);
        variable_header_vec.push((topic_name_len & 0xFF) as u8);
        for i in &self.topic_name {
            variable_header_vec.push(*i);
        }
        variable_header_vec.push((self.packet_identifier >> 8) as u8);
        variable_header_vec.push((self.packet_identifier & 0xFF) as u8);
        variable_header_vec
    }
    fn unvalue(x: Vec<u8>, readed: &mut usize) -> VariableHeaderPublish {
        let topic_name_len = (x[0] as u16) << 8 | (x[1] as u16);
        let topic_name = x[2..(2 + topic_name_len as usize)].to_vec();
        let packet_identifier = (x[2 + topic_name_len as usize] as u16) << 8 | (x[3 + topic_name_len as usize] as u16);
        *readed = 4 + topic_name_len as usize;
        VariableHeaderPublish {
            topic_name,
            packet_identifier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn unvalue_variable_header_publish() {
        let variable_header_publish = VariableHeaderPublish {
            topic_name: "Temperature".as_bytes().to_vec(),
            packet_identifier: 0x0001,
        };
        let value = variable_header_publish.value();
        // println!("value: {:?}", value);
        let mut readed = 0;
        let unvalue = VariableHeaderPublish::unvalue(value, &mut readed);
        // println!("unvalue: {:?}", unvalue);
        assert_eq!(variable_header_publish.topic_name, unvalue.topic_name);
        assert_eq!(variable_header_publish.packet_identifier, unvalue.packet_identifier);
        assert_eq!(4 + variable_header_publish.topic_name.len(), readed);
    }

    #[test]
    fn unvalue_variable_header_connack() {
        let variable_header_connack = VariableHeaderConnack {
            acknoledge_flags: 0x01,
            return_code: 0x02,
        };
        let value = variable_header_connack.value();
        let mut readed = 0;
        let unvalue = VariableHeaderConnack::unvalue(value, &mut readed);
        assert_eq!(unvalue.acknoledge_flags, variable_header_connack.acknoledge_flags);
        assert_eq!(unvalue.return_code, variable_header_connack.return_code);
        assert_eq!(2, readed);
    }

    #[test]
    fn unvalue_variable_header() {
        let protocol_name = [0x00, 0x04, b'M', b'Q', b'T', b'T'].to_vec();
        let protocol_name_stub = protocol_name.clone();
        let protocol_level = 0x04;
        let connect_flags = connect_flags::CLEAN_SESSION;
        let keep_alive = 0xFFF;
        let variable_header = VariableHeader {
            protocol_name: protocol_name,
            protocol_level: protocol_level,
            connect_flags: connect_flags,
            keep_alive: keep_alive,
        };
        let value: Vec<u8> = variable_header.value();
        // println!("{:?}", value);
        let mut readed = 0;
        let unvalue = VariableHeader::unvalue(value, &mut readed);
        // println!("{:?}", unvalue);
        assert!(unvalue.protocol_name.eq(&protocol_name_stub));
        assert_eq!(unvalue.protocol_level, protocol_level);
        assert_eq!(unvalue.connect_flags, connect_flags);
        assert_eq!(unvalue.keep_alive, keep_alive);
        assert_eq!(10, readed);
    }

    #[test]
    fn variable_header_value() {
        let protocol_name = [0x00, 0x04, b'M', b'Q', b'T', b'T'].to_vec();
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
        // println!("variable header value: {:?}", value);
        assert!(value.len() == vh_stub.len());
        assert!(vh_stub.eq(&value));
    }

    #[test]
    fn variable_header_connack_value() {
        let acknoledge_flags = 0x00;
        let return_code = connect_return::ACCEPTED;
        let vh_stub: Vec<u8> = [acknoledge_flags, return_code as u8].to_vec();
        
        let variable_header = VariableHeaderConnack {
            acknoledge_flags: acknoledge_flags,
            return_code: return_code,
        };
        let value: Vec<u8> = variable_header.value();
        println!("variable header value: {:?}", value);
        assert!(value.len() == vh_stub.len());
        assert!(vh_stub.eq(&value));
    }
}