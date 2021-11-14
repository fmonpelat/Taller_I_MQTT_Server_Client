mod header_packet;
use header_packet::{control_flags, control_type, Header, PacketHeader};
mod variable_header_packet;
use variable_header_packet::{
    connect_flags,
    VariableHeader,
    PacketVariableHeader,
    VariableHeaderConnack,
    PacketVariableHeaderConnack,
    VariableHeaderPublish,
    PacketVariableHeaderPublish,
};
mod payload_packet;
use payload_packet::{Payload, PacketPayload, PublishPayload, PacketPublishPayload };

#[derive(Debug, Default)]
pub struct Packet<T,P> {
    header: Header,
    has_variable_header: bool,
    variable_header: T,
    has_payload: bool,
    payload: P,
}

// specific implementations for each packet type
impl Packet<VariableHeader, Payload> {
    #[allow(dead_code)]
    pub fn new() -> Packet<VariableHeader, Payload> {
        Packet {
            header: Header::default(),
            has_variable_header: false,
            variable_header: VariableHeader::default(),
            has_payload: false,
            payload: Payload::default(),
        }
    }
    #[allow(dead_code)]
    fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        let variable_header = if self.has_variable_header { self.variable_header.value() } else { Vec::new() };
        let payload = if self.has_payload { self.payload.value()} else { Vec::new() };

        let vec: Vec<u8> = self.header.value().iter().cloned()
        .chain(
            variable_header.iter().cloned().chain(
                payload.iter().cloned())
        ).collect();

        for i in vec {
            res.push(i);
        }
        res
    }
}

impl Packet<VariableHeaderConnack, Payload> {
    #[allow(dead_code)]
    fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        let variable_header = if self.has_variable_header { self.variable_header.value() } else { Vec::new() };
        let payload = if self.has_payload { self.payload.value()} else { Vec::new() };

        let vec: Vec<u8> = self.header.value().iter().cloned()
        .chain(
            variable_header.iter().cloned().chain(
                payload.iter().cloned())
        ).collect();

        for i in vec {
            res.push(i);
        }
        res
    }
}

impl Packet<VariableHeaderPublish, PublishPayload> {
    #[allow(dead_code)]
    fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        let variable_header = if self.has_variable_header { self.variable_header.value() } else { Vec::new() };
        let payload = if self.has_payload { self.payload.value()} else { Vec::new() };

        let vec: Vec<u8> = self.header.value().iter().cloned()
        .chain(
            variable_header.iter().cloned().chain(
                payload.iter().cloned())
        ).collect();

        for i in vec {
            res.push(i);
        }
        res
    }
}        
// general implementation for all packets

pub trait ClientPacket {
    fn connect(&self, client_identifier: String) -> Packet<VariableHeader, Payload>;
    fn disconnect(&self) -> Packet<VariableHeader, Payload>;
    fn pingreq(&self) -> Packet<VariableHeader, Payload>;
    fn publish(&self, dup: u8, qos: u8, retain: u8, topic_name: String, message: String) -> Packet<VariableHeaderPublish, PublishPayload>;
}
impl<T, P> ClientPacket for Packet<T, P> {

    fn connect(&self, client_identifier: String) -> Packet<VariableHeader, Payload> {
        let header = Header {
            control_type: control_type::CONNECT, // 0x10
            control_flags: control_flags::RESERVED, // 0x00
            remaining_length_0: vec![0], // what remaining lenght is? (how it is calculated)
        };
        let protocol_name = [0x00, 0x04, b'M', b'Q', b'T', b'T'].to_vec();
        let variable_header:VariableHeader = VariableHeader {
            protocol_name,
            protocol_level: 0x04,
            connect_flags: connect_flags::CLEAN_SESSION, // what connect flags do i need?
            keep_alive: 0x00,
        };
        // do we need a payload if there is a connect?
        let payload = Payload {
            client_identifier,
            ..Payload::default()
        };

        // building the struct packet
        let mut packet = Packet {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: true,
            payload,
        };
        let remaining_length = (packet.variable_header.value().len() + packet.payload.value().len()) as u32;
        // println!("calculated packet remaining length: {}", remaining_length);
        packet.header.set_remaining_length(remaining_length);
        // println!(" decoded remaining length {:?}", packet.header.decode_remaining_length());
        packet
    }

    fn disconnect(&self) -> Packet<VariableHeader, Payload> {
        let header = Header {
            control_type: control_type::DISCONNECT,
            control_flags: control_flags::RESERVED,
            remaining_length_0: vec![0],
        };
        // building the struct packet
        Packet {
            header,
            has_variable_header: false,
            variable_header: VariableHeader::default(),
            has_payload: false,
            payload: Payload::default(),
        }
    }

    fn pingreq(&self) -> Packet<VariableHeader, Payload> {
        let header = Header {
            control_type: control_type::PINGREQ,
            control_flags: control_flags::RESERVED,
            remaining_length_0: vec![0],
        };
        // building the struct packet
        return Packet {
            header,
            has_variable_header: false,
            variable_header: VariableHeader::default(),
            has_payload: false,
            payload: Payload::default(),
        };
    }

    fn publish(&self, dup: u8, qos: u8, retain: u8, topic_name: String, message: String) -> Packet<VariableHeaderPublish, PublishPayload> {
        let header = Header {
            control_type: control_type::PUBLISH,
            control_flags: dup | qos | retain,
            remaining_length_0: vec![0],
        };
        let topic_name_len = topic_name.len() as u16;
        let mut topic_name_bytes:Vec<u8> = vec![(topic_name_len>>8) as u8, (topic_name_len & 0xFF) as u8];
        topic_name_bytes.extend(topic_name.bytes());

        let variable_header = VariableHeaderPublish {
            topic_name: topic_name_bytes,
            packet_identifier: 10, // TODO: is this conformed to 3.1.1???
        };

        let payload = PublishPayload {
            message
        };
        // building the struct packet
        let mut packet = Packet {
            header,
            has_variable_header: true,
            variable_header: variable_header,
            has_payload: true,
            payload,
        };
        let remaining_length = (packet.variable_header.value().len() + packet.payload.value().len()) as u32;
        packet.header.set_remaining_length(remaining_length);
        return packet;
    }
}

pub trait ServerPacket {
    fn connack(&self, connect_ack_flags: u8, connect_return: u8 ) -> Packet<VariableHeaderConnack, Payload>;
    fn pingresp(&self) -> Packet<VariableHeader, Payload>;
}

impl<T, P> ServerPacket for Packet<T, P> {

    fn connack(&self, connect_ack_flags: u8, connect_return: u8) -> Packet<VariableHeaderConnack, Payload> {
        let variable_header: VariableHeaderConnack = VariableHeaderConnack {
            acknoledge_flags: connect_ack_flags,
            return_code: connect_return,
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

        // TODO: check if this function can modify the self packet
        // self.header = header;
        // self.variable_header = variable_header;
        // self.payload = payload;

        Packet {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: true,
            payload,
        }
    }

    fn pingresp(&self) -> Packet<VariableHeader, Payload> {
        let header = Header {
            control_type: control_type::PINGRESP,
            control_flags: control_flags::RESERVED,
            remaining_length_0: vec![0],
        };
        // building the struct packet
        return Packet {
            header,
            has_variable_header: false,
            variable_header: VariableHeader::default(),
            has_payload: false,
            payload: Payload::default(),
        };
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    mod packets {
        use crate::mqtt_packet_service::variable_header_packet::{connect_ack_flags, connect_return};

        use super::*;
        #[test]
        fn check_connect_packet() {
            let connect_head_stub = vec![0x10, 18, 0, 4, 77, 81, 84, 84, 4, 2, 0, 0];
            let client_identifier = String::from("testId");
            let connect_stub: Vec<u8> = connect_head_stub.iter().copied().chain(
                (client_identifier.len() as u16).to_be_bytes().iter().copied().chain(
                    client_identifier.as_bytes().iter().copied()
                )
            ).collect();
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.connect(client_identifier);
            let value = packet.value();
            // println!("value connect: {:?}", value);
            // println!("connect stub: {:?}", connect_stub);
            assert_eq!(value.len(), connect_stub.len());
            assert!(connect_stub.eq(&value));
        }

        #[test]
        fn check_connack_packet() {
            let header = vec![0x20, 0x02];
            let mut variable_header = Vec::with_capacity(2);
            variable_header.push(connect_ack_flags::SESSION_PRESENT);
            variable_header.push(connect_return::ACCEPTED);
            let connack_head_stub:Vec<u8> = header.iter().copied().chain(
                variable_header.iter().copied()
            ).collect();
           
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.connack(connect_ack_flags::SESSION_PRESENT, connect_return::ACCEPTED);
            let value = packet.value();
            // println!("value connack: {:?}", value);
            // println!("connack stub: {:?}", connack_head_stub);
            assert_eq!(value.len(), connack_head_stub.len());
            assert!(connack_head_stub.eq(&value));
        }

        #[test]
        fn check_disconnect_packet() {
            let header = vec![14, 0x00];
            let disconnect_stub: Vec<u8> = header.iter().copied().collect();
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.disconnect();
            let value = packet.value();
            // println!("value disconnect: {:?}", value);
            // println!("disconnect stub: {:?}", disconnect_stub);
            assert!(value.len() == disconnect_stub.len());
            assert!(disconnect_stub.eq(&value));
        }

        #[test]
        fn check_pingreq_packet() {
            let header = vec![12, 0x00];
            let pingreq_stub: Vec<u8> = header.iter().copied().collect();
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.pingreq();
            let value = packet.value();
            // println!("value pingreq: {:?}", value);
            // println!("pingreq stub: {:?}", pingreq_stub);
            assert!(value.len() == pingreq_stub.len());
            assert!(pingreq_stub.eq(&value));
        }

        #[test]
        fn check_publish_packet() {
            let dup = control_flags::DUP;
            let qos = control_flags::QOS0;
            let retain = control_flags::RETAIN;
            let header = vec![control_type::PUBLISH + ((dup | qos | retain) as u8), 24]; // length of 24 for this example
            let topic_name = String::from("testTopic");
            let packet_identifier:Vec<u8> = vec![0, 10];
            let payload = String::from("testPayload");
            let publish_stub: Vec<u8> = header.iter().copied().chain(
                (topic_name.len() as u16).to_be_bytes().iter().copied().chain(
                    topic_name.as_bytes().iter().copied().chain(
                        packet_identifier.iter().copied().chain(
                            payload.as_bytes().iter().copied()
                        )
                    )
                )
            ).collect();
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.publish(dup, qos, retain, topic_name, payload);
            let value = packet.value();
            // println!("value publish: {:?}", value);
            // println!("publish stub: {:?}", publish_stub);
            assert!(value.len() == publish_stub.len());
            assert!(publish_stub.eq(&value));
        }
    }
}