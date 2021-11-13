mod header_packet;
use header_packet::{control_flags, control_type, Header, PacketHeader};
mod variable_header_packet;
use variable_header_packet::{connect_flags, connect_return, VariableHeader, PacketVariableHeader, VariableHeaderConnack, PacketVariableHeaderConnack};
mod payload_packet;
use payload_packet::{Payload, PacketPayload};

#[derive(Debug, Default)]
pub struct Packet<T> {
    header: Header,
    variable_header: T,
    payload: Payload,
}

// specific implementations for each packet type
impl Packet<VariableHeader> {
    pub fn new() -> Packet<VariableHeader> {
        Packet {
            header: Header::default(),
            variable_header: VariableHeader::default(),
            payload: Payload::default(),
        }
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
}

impl Packet<VariableHeaderConnack> {
    pub fn new() -> Packet<VariableHeaderConnack> {
        Packet {
            header: Header::default(),
            variable_header: VariableHeaderConnack::default(),
            payload: Payload::default(),
        }
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
}

pub trait ModPacket {
    fn connect(&self, client_identifier: String) -> Packet<VariableHeader>;
    fn connack(&self) -> Packet<VariableHeaderConnack>;
}
// general implementation for all packets
impl<T> ModPacket for Packet<T> {

    fn connect(&self, client_identifier: String) -> Packet<VariableHeader> {
        let header = Header {
            control_type: control_type::CONNECT, // 0x10
            control_flags: control_flags::RESERVED, // 0x00
            remaining_length_0: vec![0], // what remaining lenght is? (how it is calculated)
        };
        let protocol_name = [0x00, 0x04, b'M', b'Q', b'T', b'T'].to_vec();
        let variable_header:VariableHeader = VariableHeader {
            protocol_name: protocol_name,
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
        let mut packet = Packet {
            header: header,
            variable_header: variable_header,
            payload: payload,
        };
        let remaining_length = (packet.variable_header.value().len() + packet.payload.value().len()) as u32;
        println!("calculated packet remaining length: {}", remaining_length);
        packet.header.set_remaining_length(remaining_length);
        println!(" decoded remaining length {:?}", packet.header.decode_remaining_length());
        return packet;
    }

    fn connack(&self) -> Packet<VariableHeaderConnack> {
        let variable_header: VariableHeaderConnack = VariableHeaderConnack {
            acknoledge_flags: 0x00,
            return_code: connect_return::ACCEPTED,
        };
        let mut header = Header {
            control_type: control_type::CONNACK,
            control_flags: control_flags::RESERVED,
            remaining_length_0: vec![0],
        };
        // empty payload
        let payload = Payload {
            ..Payload::default()
        };
        let remaining_length = (variable_header.value().len() + payload.value().len()) as u32;
        header.set_remaining_length(remaining_length);

        let packet = Packet {
            header: header,
            variable_header: variable_header,
            payload: payload,
        };
        let _header = packet.header.value();
        println!("header: {:?}", _header);
        let _variable_header = packet.variable_header.value();
        println!("variable header: {:?}", _variable_header);
        let _payload = packet.payload.value();
        println!("payload: {:?}", _payload);

        println!("decoded packet remaining length: {}", packet.header.decode_remaining_length());
        return packet;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod packets {
        use super::*;
        #[test]
        fn connect_packet() {
            let connect_head_stub = vec![0x10, 0, 0, 0, 18, 0, 4, 77, 81, 84, 84, 4, 2, 0, 0];
            let client_identifier = String::from("testId");
            let connect_stub: Vec<u8> = connect_head_stub.iter().copied().chain(
                (client_identifier.len() as u16).to_be_bytes().iter().copied().chain(
                    client_identifier.as_bytes().iter().copied()
                )
            ).collect();
            let packet = Packet::<VariableHeader>::new();
            let packet = packet.connect(client_identifier);
            let value = packet.value();
            println!("value connect: {:?}", value);
            println!("connect stub: {:?}", connect_stub);
            assert_eq!(value.len(), connect_stub.len());
            assert!(connect_stub.eq(&value));
        }

        #[test]
        fn connack_packet() {
            let header = vec![0x20, 0x02];
            let variable_header = vec![0, 0, 2, 0, 0];
            let connack_head_stub:Vec<u8> = header.iter().copied().chain(
                variable_header.iter().copied()
            ).collect();
           
            let packet = Packet::<VariableHeader>::new();
            packet.value();
            //let packet = Packet::<VariableHeaderConnack>::new();

            let packet = packet.connack();
            let value = packet.value();
            println!("value connack: {:?}", value);
            println!("connack stub: {:?}", connack_head_stub);
            assert_eq!(value.len(), connack_head_stub.len());
            // assert!(connect_stub.eq(&value));
        }
    }
}
