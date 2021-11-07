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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn header_connect_value() {
        let control_type = control_type::CONNECT; // 0x10
        let control_flags = control_flags::RESERVED; // 0x00
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
}