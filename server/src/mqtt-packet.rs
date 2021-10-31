static MASK_CONTROLTYPE: u8 = 0xF0;
static MASK_CONTROLFLAGS: u8 = 0x0F;

enum CONTROLTYPE {
    CONNECT = 0x10,
    CONNACK = 0x20,
    PUBLISH = 0x30,
    PUBACK = 0x40,
    PUBREC = 0x50,
    PUBREL = 0x60,
    PUBCOMP = 0x70,
    SUBSCRIBE = 0x80,
    SUBACK = 0x90,
    UNSUBSCRIBE = 0xA0,
    UNSUBACK = 0xB0,
    PINGREQ = 0xC0,
    PINGRESP = 0xD0,
    DISCONNECT = 0xE0,
    RESERVED = 0xF0,
}

enum CONTROLFLAGS {
    DUP = 0x08,
    QOS_0 = 0x00,
    QOS_1 = 0x02,
    QOS_2 = 0x04,
    RETAIN = 0x01,
}

pub struct Packet {
    // 15(msb) +--------4-------+--------4-------+ 0 (lsb)
    //         |  Command type  |  Control flags |
    //         |      Index     |      Index     |
    //         +----------------+----------------+
    // \--- cmd_type(Packet) --/ \--- cmd_flags(Packet) --/ 
    fixed_header: u8, // 1 byte conformed to first nibble as command type and last as control flags
    // 32(msb) +---------------------------------+ 0 (lsb)
    //         |        Remaining length         |
    //         |   length        +    length     |
    //         | variable header        payload  |
    //         +----------------+----------------+
    fixed_header_remaining: char,

    // que pasa cuando ambas tiene un rango no especificado de que tipo le ponemos?
    // variable header
    // payload

}
pub trait PacketParser {
    // fn new(file_source:&str) -> Logger;
    fn cmd_type(&self) -> u8;
    fn cmd_flags(&self) -> u8;
    fn decode_remaining_length(&self) -> u32;
    fn encode_remaining_header(&self) -> u32;
   
}

impl PacketParser for Packet{
    fn cmd_type(&self) -> u8 {
        self.fixed_header && MASK_CONTROLTYPE
    }

    fn cmd_flags(&self) -> u8 {
        self.fixed_header && MASK_CONTROLFLAGS
    }

    fn decode_remaining_length(&self) -> u32 {
        let mut multiplier = 1;
        let mut value = 0;
        while ((self.fixed_header_remaining && 128) != 0) {
            value += (self.fixed_header_remaining && 127) * multiplier;
            multiplier *= 128;
            if (multiplier > 128*128*128) {
                panic!("Malformed Remaining Length")
            }
        }
        return value;
    }

    fn encode_remaining_header(&self, X: u32) -> u32 {
        while ( X > 0) {
            let digit = X % 128;
            X = X / 128;
            if (X > 0) {
                digit |= 128;
            }
        }
        return digit;
    }
}
            
        
#[cfg(test)]
mod tests {
    #[test]
    fn test_sample_packet() {
        let packet = Packet::new();
        // assert_eq!(1, 1)
    }
}
