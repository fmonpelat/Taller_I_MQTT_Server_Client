
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

pub mod connect_return {
    pub const ACCEPTED: u8 = 0x00;
    pub const UNACCEPTABLE_PROTOCOL_VERSION: u8 = 0x01;
    pub const IDENTIFIER_REJECTED: u8 = 0x02;
    pub const SERVER_UNAVAILABLE: u8 = 0x03;
    pub const BAD_USERNAME_OR_PASSWORD: u8 = 0x04;
    pub const NOT_AUTHORIZED: u8 = 0x05; 
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
        return variable_header_vec;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
}