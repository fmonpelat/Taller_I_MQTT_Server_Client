
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
    pub remaining_length_0: Vec<u8>, // 4 byte
}
// traits and impl for header
pub trait PacketHeader {
    fn get_cmd_type(&self) -> u8;
    fn get_cmd_flags(&self) -> u8;
    fn decode_remaining_length(&self) -> u32;
    fn encode_remaining_length(&self, x: u32) -> Vec<u8>;
    fn set_remaining_length(&mut self, x: u32);
    fn value(&self) -> Vec<u8>;
}

impl PacketHeader for Header {

    fn value(&self) -> Vec<u8> {
        let mut header_vec: Vec<u8> = Vec::with_capacity(3);
        header_vec.push(self.get_cmd_type() + self.get_cmd_flags());
        for i in self.remaining_length_0.iter() {
            header_vec.push(*i);
        }
        return header_vec;
    }

    fn get_cmd_type(&self) -> u8 {
        self.control_type
    }

    fn get_cmd_flags(&self) -> u8 {
        self.control_flags
    }

    fn decode_remaining_length(&self) -> u32 {
        // multiplier = 1
        // 300 value = 0
        // 301 do
        // 302 encodedByte = 'next byte from stream'
        // 303 value += (encodedByte AND 127) * multiplier
        // 304 multiplier *= 128
        // 305 if (multiplier > 128*128*128)
        // 306 throw Error(Malformed Remaining Length)
        // 307 while ((encodedByte AND 128) != 0)
        let mut array = self.remaining_length_0.clone();
        array.reverse();
        let mut multiplier: u32 = 1;
        let mut value = 0;
        loop {
            let encoded_byte: u8 = array.pop().unwrap();
            {
                value += (encoded_byte & 127) as u32 * multiplier;
                if multiplier > 128*128*128*128 {
                    panic!("Malformed Remaining Length")
                }
                multiplier *= 128;
            };
            if (encoded_byte & 128) == 0 { break }
        }
        
        return value as u32;
    }

    fn encode_remaining_length(&self, x: u32) -> Vec<u8> {
        // do
        // 284 encodedByte = X MOD 128
        // 285 X = X DIV 128
        // 286 // if there are more data to encode, set the top bit of this byte
        // 287 if ( X > 0 )
        // 288 encodedByte = encodedByte OR 128
        // 289 endif
        // 290 'output' encodedByte
        // 291 while ( X > 0 )
        let mut x_: u32 = x.clone();
        let mut return_vec = Vec::with_capacity(4);
        loop {
            {
                let mut encoded_byte: u8 = (x_ % 128) as u8;
                x_ = x_ / 128;
                if x_ > 0 {
                    encoded_byte = encoded_byte | 128; // set the topmost bit to 1
                }
                return_vec.push(encoded_byte);
            };
            if x_ <= 0 { break }
        }
        return return_vec;
    }

    fn set_remaining_length(&mut self, x: u32) {
        let vector= self.encode_remaining_length(x);
        self.remaining_length_0 = vector.clone();
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;
    
    #[test]
    fn check_encode_remaining_len() {
        let header = Header::default();
        let vector = header.encode_remaining_length(400);
        assert!(vector.len() == 2);
        assert!(vector[0] == 144 && vector[1] == 3);

        let vector = header.encode_remaining_length(16384);
        assert!(vector.len() == 3);
        assert!(vector[0] == 128 && vector[1] == 128 && vector[2] == 1);

        let vector = header.encode_remaining_length(2097152);
        assert!(vector.len() == 4);
        assert!(vector[0] == 128 && vector[1] == 128 && vector[2] == 128 && vector[3] == 1);
    }

    #[test]
    fn check_set_remaining_len() {
        let mut header = Header::default();
        header.set_remaining_length(100);
        assert!(header.remaining_length_0.len() == 1);

        let value = header.decode_remaining_length();
        assert!(value == 100);

        header.set_remaining_length(2097152);
        assert!(header.remaining_length_0.len() == 4);

        let value = header.decode_remaining_length();
        assert!(value == 2097152);
    }

    #[test]
    fn check_remaining_len_upperbounds() {
        panic::set_hook(Box::new(|_info| {})); 
        let mut header = Header::default();
        header.set_remaining_length(268435456);
        let result = panic::catch_unwind( move || header.decode_remaining_length());
        assert!(result.is_err());
    }

    #[test]
    fn header_remaining_len() {
        let mut header = Header::default();
        header.set_remaining_length(0);  // 0 bytes length
        assert_eq!(header.decode_remaining_length(), 0);
       
        header.set_remaining_length(127);  // 1 bytes length
        assert_eq!(header.decode_remaining_length(), 127);
        
        header.set_remaining_length(128); // 2 bytes length
        assert_eq!(header.decode_remaining_length(), 128);

        header.set_remaining_length(129); // 2 bytes length
        assert_eq!(header.decode_remaining_length(), 129);

        header.set_remaining_length(128*128); // 3 bytes length
        assert_eq!(header.decode_remaining_length(), 128*128);

        header.set_remaining_length(128*128*128); // 4 bytes length
        assert_eq!(header.decode_remaining_length(), 128*128*128);
    }

    #[test]
    fn header_connect_value() {
        let control_type = control_type::CONNECT; // 0x10
        let control_flags = control_flags::RESERVED; // 0x00
        let remaining_length_0 = vec![0];
        let header_stub = vec![
            control_type + control_flags,
            remaining_length_0[0],
        ];
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