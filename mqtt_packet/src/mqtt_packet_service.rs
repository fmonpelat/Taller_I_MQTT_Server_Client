pub mod header_packet;
use header_packet::{control_flags, control_type, control_type_vec, Header, PacketHeader};
pub mod variable_header_packet;
use variable_header_packet::{
    connect_flags, PacketVariableHeader, PacketVariableHeaderConnack,
    PacketVariableHeaderPacketIdentifier, PacketVariableHeaderPublish, VariableHeader,
    VariableHeaderConnack, VariableHeaderPacketIdentifier, VariableHeaderPublish,
};
pub mod payload_packet;
use payload_packet::{
    PacketPayload, PacketPayloadSubscribe, PacketPublishPayload, PacketSubackPayload,
    PacketUnsubscribePayload, Payload, PublishPayload, SubscribePayload, UnsubscribePayload,
};

use self::payload_packet::SubackPayload;

/// Implementation of the MQTT packet service.
/// This service is used to create and parse MQTT packets

#[derive(Debug, Default)]
pub struct Packet<T, P> {
    pub header: Header,
    pub has_variable_header: bool,
    pub variable_header: T,
    pub has_payload: bool,
    pub payload: P,
}

// specific implementations for each packet type
/// Packet used by connect method
impl Packet<VariableHeader, Payload> {
    /// Creates a new Packet<VariableHeader, Payload>
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
    /// Deserializes a Packet<VariableHeader, Payload>
    pub fn unvalue(x: Vec<u8>) -> Packet<VariableHeader, Payload> {
        let mut absolute_index: usize = 0;
        let mut readed: usize = 0;
        let mut has_variable_header: bool = false;
        let mut has_payload: bool = false;
        let mut variable_header: VariableHeader = VariableHeader::default();
        let mut payload: Payload = Payload::default();
        let header = Header::unvalue(x.clone(), &mut readed);
        absolute_index += readed;

        if absolute_index < x.len() {
            variable_header =
                VariableHeader::unvalue(x[absolute_index..x.len()].to_vec(), &mut readed);
            if readed > 0 {
                has_variable_header = true;
            }
            absolute_index += readed;
        }
        if absolute_index < x.len() {
            payload = Payload::unvalue(x[absolute_index..x.len()].to_vec(), &mut readed);
            if readed > 0 {
                has_payload = true;
            }
        }
        Packet::<VariableHeader, Payload> {
            header,
            has_variable_header,
            variable_header,
            has_payload,
            payload,
        }
    }
    /// Serializes a Packet<VariableHeader, Payload>
    #[allow(dead_code)]
    pub fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        let variable_header = if self.has_variable_header {
            self.variable_header.value()
        } else {
            Vec::new()
        };
        let payload = if self.has_payload {
            self.payload.value()
        } else {
            Vec::new()
        };

        let vec: Vec<u8> = self
            .header
            .value()
            .iter()
            .cloned()
            .chain(
                variable_header
                    .iter()
                    .cloned()
                    .chain(payload.iter().cloned()),
            )
            .collect();

        for i in vec {
            res.push(i);
        }
        res
    }
}

impl Packet<VariableHeaderConnack, Payload> {
    /// Creates a new Packet<VariableHeaderConnack, Payload>
    #[allow(dead_code)]
    pub fn new() -> Packet<VariableHeaderConnack, Payload> {
        Packet {
            header: Header::default(),
            has_variable_header: false,
            variable_header: VariableHeaderConnack::default(),
            has_payload: false,
            payload: Payload::default(),
        }
    }
    /// Serializes a new Packet<VariableHeaderConnack, Payload>
    #[allow(dead_code)]
    pub fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        let variable_header = if self.has_variable_header {
            self.variable_header.value()
        } else {
            Vec::new()
        };
        let payload = if self.has_payload {
            self.payload.value()
        } else {
            Vec::new()
        };

        let vec: Vec<u8> = self
            .header
            .value()
            .iter()
            .cloned()
            .chain(
                variable_header
                    .iter()
                    .cloned()
                    .chain(payload.iter().cloned()),
            )
            .collect();

        for i in vec {
            res.push(i);
        }
        res
    }

    /// Deserializes a Packet<VariableHeaderConnack, Payload>
    pub fn unvalue(x: Vec<u8>) -> Packet<VariableHeaderConnack, Payload> {
        let mut absolute_index: usize = 0;
        let mut readed: usize = 0;
        let mut has_variable_header = false;
        let mut has_payload = false;
        let header = Header::unvalue(x.clone(), &mut readed);
        absolute_index += readed;
        let variable_header =
            VariableHeaderConnack::unvalue(x[absolute_index..x.len()].to_vec(), &mut readed);
        if readed > 0 {
            has_variable_header = true;
        }
        absolute_index += readed;

        let payload = Payload::unvalue(x[absolute_index..x.len()].to_vec(), &mut readed);
        if readed > 0 {
            has_payload = true;
        }
        Packet::<VariableHeaderConnack, Payload> {
            header,
            has_variable_header,
            variable_header,
            has_payload,
            payload,
        }
    }
}

impl Packet<VariableHeaderPacketIdentifier, Payload> {
    /// Creates a new Packet<VariableHeaderPublishAck, Payload>
    #[allow(dead_code)]
    pub fn new() -> Packet<VariableHeaderPacketIdentifier, Payload> {
        Packet {
            header: Header::default(),
            has_variable_header: false,
            variable_header: VariableHeaderPacketIdentifier::default(),
            has_payload: false,
            payload: Payload::default(),
        }
    }

    /// Deserializes a Packet<VariableHeaderPublishAck, Payload>
    #[allow(dead_code)]
    pub fn unvalue(x: Vec<u8>) -> Packet<VariableHeaderPacketIdentifier, Payload> {
        let mut absolute_index: usize = 0;
        let mut readed: usize = 0;
        let header = Header::unvalue(x.clone(), &mut readed);
        absolute_index += readed;

        let variable_header = VariableHeaderPacketIdentifier::unvalue(
            x[absolute_index..x.len()].to_vec(),
            &mut readed,
        );
        Packet::<VariableHeaderPacketIdentifier, Payload> {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: false,
            payload: Payload::default(),
        }
    }

    /// Serializes a Packet<VariableHeaderPublishAck, Payload>
    #[allow(dead_code)]
    pub fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        let variable_header = if self.has_variable_header {
            self.variable_header.value()
        } else {
            Vec::new()
        };
        let mut payload = if self.has_payload {
            self.payload.value()
        } else {
            Vec::new()
        };
        // put payload len before payload content
        if !payload.is_empty() {
            let payload_len = payload.len() as u16;
            let mut vec: Vec<u8> = Vec::with_capacity(payload.len() + 1);
            vec.push((payload_len >> 8) as u8);
            vec.push((payload_len & 0xFF) as u8);
            for i in payload {
                vec.push(i);
            }
            payload = vec;
        }

        let vec: Vec<u8> = self
            .header
            .value()
            .iter()
            .cloned()
            .chain(
                variable_header
                    .iter()
                    .cloned()
                    .chain(payload.iter().cloned()),
            )
            .collect();

        for i in vec {
            res.push(i);
        }
        res
    }
}

impl Packet<VariableHeaderPublish, PublishPayload> {
    /// Creates a new Packet<VariableHeaderPublish, Payload>
    #[allow(dead_code)]
    pub fn new() -> Packet<VariableHeaderPublish, PublishPayload> {
        Packet {
            header: Header::default(),
            has_variable_header: true,
            variable_header: VariableHeaderPublish::default(),
            has_payload: true,
            payload: PublishPayload::default(),
        }
    }

    /// Serializes a Packet<VariableHeaderPublish, PublishPayload>
    #[allow(dead_code)]
    pub fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(1024);
        let variable_header = if self.has_variable_header {
            self.variable_header.value()
        } else {
            Vec::new()
        };
        let payload = if self.has_payload {
            self.payload.value()
        } else {
            Vec::new()
        };

        let vec: Vec<u8> = self
            .header
            .value()
            .iter()
            .cloned()
            .chain(
                variable_header.iter().cloned().chain(
                    (payload.len() as u16)
                        .to_be_bytes()
                        .iter()
                        .cloned()
                        .chain(payload.iter().cloned()),
                ),
            )
            .collect();

        for i in vec {
            res.push(i);
        }
        res
    }

    /// Deserializes a Packet<VariableHeaderPublish, Payload>
    #[allow(dead_code)]
    pub fn unvalue(x: Vec<u8>) -> Packet<VariableHeaderPublish, PublishPayload> {
        let mut absolute_index: usize = 0;
        let mut has_payload = false;
        let mut has_variable_header = false;
        let mut readed: usize = 0;
        let header = Header::unvalue(x.clone(), &mut readed);
        absolute_index += readed;

        let variable_header =
            VariableHeaderPublish::unvalue(x[absolute_index..x.len()].to_vec(), &mut readed);

        if readed > 0 {
            has_variable_header = true;
        }
        absolute_index += readed;

        let payload = PublishPayload::unvalue(x[absolute_index..x.len()].to_vec(), &mut readed);
        if readed > 0 {
            has_payload = true;
        }
        Packet::<VariableHeaderPublish, PublishPayload> {
            header,
            has_variable_header,
            variable_header,
            has_payload,
            payload,
        }
    }
}

impl Packet<VariableHeaderPacketIdentifier, SubscribePayload> {
    /// Creates a new Packet<VariableHeaderPacketIdentifier, SubscribePayload>
    #[allow(dead_code)]
    pub fn new() -> Packet<VariableHeaderPacketIdentifier, SubscribePayload> {
        Packet {
            header: Header::default(),
            has_variable_header: true,
            variable_header: VariableHeaderPacketIdentifier::default(),
            has_payload: true,
            payload: SubscribePayload::default(),
        }
    }

    /// Serializes a Packet<VariableHeaderPacketIdentifier, SubscribePayload>
    #[allow(dead_code)]
    pub fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(3072); // max 3KB packet
        let variable_header = if self.has_variable_header {
            self.variable_header.value()
        } else {
            Vec::new()
        };
        let payload = if self.has_payload {
            self.payload.value()
        } else {
            Vec::new()
        };

        let vec: Vec<u8> = self
            .header
            .value()
            .iter()
            .cloned()
            .chain(
                variable_header
                    .iter()
                    .cloned()
                    .chain(payload.iter().cloned()),
            )
            .collect();

        for i in vec {
            res.push(i);
        }
        res
    }

    /// Deserializes a Packet<VariableHeaderPacketIdentifier, SubscribePayload>
    #[allow(dead_code)]
    pub fn unvalue(x: Vec<u8>) -> Packet<VariableHeaderPacketIdentifier, SubscribePayload> {
        let mut absolute_index: usize = 0;
        let mut has_payload = false;
        let mut has_variable_header = false;
        let mut readed: usize = 0;
        let header = Header::unvalue(x.clone(), &mut readed);
        // sum all elements of header.remaining_length_0 as u16
        let remaining_len: u16 = header
            .remaining_length_0
            .iter()
            .fold(0, |acc: u16, x| acc + *x as u16);
        let packet_length: usize = header.remaining_length_0.len() + 2 + remaining_len as usize;

        absolute_index += readed;

        let variable_header = VariableHeaderPacketIdentifier::unvalue(
            x[absolute_index..x.len()].to_vec(),
            &mut readed,
        );

        if readed > 0 {
            has_variable_header = true;
        }
        absolute_index += readed;

        let payload =
            SubscribePayload::unvalue(x[absolute_index..(packet_length - 1)].to_vec(), &mut readed);
        if readed > 0 {
            has_payload = true;
        }
        Packet::<VariableHeaderPacketIdentifier, SubscribePayload> {
            header,
            has_variable_header,
            variable_header,
            has_payload,
            payload,
        }
    }
}

impl Packet<VariableHeaderPacketIdentifier, SubackPayload> {
    /// Creates a new Packet<VariableHeaderPacketIdentifier, SubackPayload>
    #[allow(dead_code)]
    pub fn new() -> Packet<VariableHeaderPacketIdentifier, SubackPayload> {
        Packet {
            header: Header::default(),
            has_variable_header: true,
            variable_header: VariableHeaderPacketIdentifier::default(),
            has_payload: true,
            payload: SubackPayload::default(),
        }
    }

    /// Serializes a Packet<VariableHeaderPacketIdentifier, SubackPayload>
    #[allow(dead_code)]
    pub fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(3072); // max 3KB packet
        let variable_header = if self.has_variable_header {
            self.variable_header.value()
        } else {
            Vec::new()
        };
        let payload = if self.has_payload {
            self.payload.value()
        } else {
            Vec::new()
        };

        let vec: Vec<u8> = self
            .header
            .value()
            .iter()
            .cloned()
            .chain(
                variable_header
                    .iter()
                    .cloned()
                    .chain(payload.iter().cloned()),
            )
            .collect();

        for i in vec {
            res.push(i);
        }
        res
    }

    /// Deserializes a Packet<VariableHeaderPacketIdentifier, SubackPayload>
    #[allow(dead_code)]
    pub fn unvalue(x: Vec<u8>) -> Packet<VariableHeaderPacketIdentifier, SubackPayload> {
        let mut absolute_index: usize = 0;
        let mut has_payload = false;
        let mut has_variable_header = false;
        let mut readed: usize = 0;
        let header = Header::unvalue(x.clone(), &mut readed);
        absolute_index += readed;

        let variable_header = VariableHeaderPacketIdentifier::unvalue(
            x[absolute_index..x.len()].to_vec(),
            &mut readed,
        );

        if readed > 0 {
            has_variable_header = true;
        }
        absolute_index += readed;

        let payload = SubackPayload::unvalue(x[absolute_index..x.len()].to_vec(), &mut readed);
        if readed > 0 {
            has_payload = true;
        }
        Packet::<VariableHeaderPacketIdentifier, SubackPayload> {
            header,
            has_variable_header,
            variable_header,
            has_payload,
            payload,
        }
    }
}

impl Packet<VariableHeaderPacketIdentifier, UnsubscribePayload> {
    /// Creates a new Packet<VariableHeaderPacketIdentifier, UnsubscribePayload>
    #[allow(dead_code)]
    pub fn new() -> Packet<VariableHeaderPacketIdentifier, UnsubscribePayload> {
        Packet {
            header: Header::default(),
            has_variable_header: true,
            variable_header: VariableHeaderPacketIdentifier::default(),
            has_payload: true,
            payload: UnsubscribePayload::default(),
        }
    }

    /// Serializes a Packet<VariableHeaderPacketIdentifier, UnsubscribePayload>
    #[allow(dead_code)]
    pub fn value(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::with_capacity(3072); // max 3KB packet
        let variable_header = if self.has_variable_header {
            self.variable_header.value()
        } else {
            Vec::new()
        };
        let payload = if self.has_payload {
            self.payload.value()
        } else {
            Vec::new()
        };

        let vec: Vec<u8> = self
            .header
            .value()
            .iter()
            .cloned()
            .chain(
                variable_header
                    .iter()
                    .cloned()
                    .chain(payload.iter().cloned()),
            )
            .collect();

        for i in vec {
            res.push(i);
        }
        res
    }

    /// Deserializes a Packet<VariableHeaderPacketIdentifier, UnsubscribePayload>
    #[allow(dead_code)]
    pub fn unvalue(x: Vec<u8>) -> Packet<VariableHeaderPacketIdentifier, UnsubscribePayload> {
        let mut absolute_index: usize = 0;
        let mut has_payload = false;
        let mut has_variable_header = false;
        let mut readed: usize = 0;
        let header = Header::unvalue(x.clone(), &mut readed);
        // sum all elements of header.remaining_length_0 as u16
        let remaining_len: u16 = header
            .remaining_length_0
            .iter()
            .fold(0, |acc: u16, x| acc + *x as u16);
        let packet_length: usize = header.remaining_length_0.len() + 2 + remaining_len as usize;

        absolute_index += readed;

        let variable_header = VariableHeaderPacketIdentifier::unvalue(
            x[absolute_index..x.len()].to_vec(),
            &mut readed,
        );

        if readed > 0 {
            has_variable_header = true;
        }
        absolute_index += readed;

        let payload = UnsubscribePayload::unvalue(
            x[absolute_index..(packet_length - 1)].to_vec(),
            &mut readed,
        );
        if readed > 0 {
            has_payload = true;
        }
        Packet::<VariableHeaderPacketIdentifier, UnsubscribePayload> {
            header,
            has_variable_header,
            variable_header,
            has_payload,
            payload,
        }
    }
}
// general implementation for all packets
pub trait Utils {
    fn get_packet_length(vec: &[u8], readed: &mut usize) -> usize;
    fn is_mqtt_packet(vec: &[u8]) -> bool;
}

impl<T, P> Utils for Packet<T, P> {
    fn is_mqtt_packet(vec: &[u8]) -> bool {
        if vec.len() < 2 {
            return false;
        }
        let header = vec[0];
        let control_type = header & 0xF0;
        if control_type == 0x00 {
            return false;
        }
        control_type_vec::CONTROL_TYPE.contains(&control_type)
    }
    fn get_packet_length(vec: &[u8], readed: &mut usize) -> usize {
        *readed = 0;
        let remaining_len = Header::get_remaining_length(vec.to_vec(), readed);
        // println!("remaining_len: {:?} readed: {:?}", remaining_len, readed);
        let remaining = Header::decode_remaining_length(&remaining_len);
        // println!("decoded remaining: {}", remaining);
        remaining as usize
    }
}

pub trait ClientPacket {
    fn connect(
        &self,
        client_identifier: String,
        clean_session: bool,
        will_topic: String,
        will_message: String,
    ) -> Packet<VariableHeader, Payload>;
    fn connect_with_credentials(
        &self,
        client_identifier: String,
        username: String,
        password: String,
        clean_session: bool,
        will_topic: String,
        will_message: String,
    ) -> Packet<VariableHeader, Payload>;
    fn disconnect(&self) -> Packet<VariableHeader, Payload>;
    fn pingreq(&self) -> Packet<VariableHeader, Payload>;
    fn puback(&self, packet_identifier: u16) -> Packet<VariableHeaderPacketIdentifier, Payload>;
    fn publish(
        &self,
        dup: u8,
        qos: u8,
        retain: u8,
        packet_identifier: u16,
        topic_name: String,
        message: String,
    ) -> Packet<VariableHeaderPublish, PublishPayload>;
    fn subscribe(
        &self,
        packet_identifier: u16,
        topic_names: Vec<String>,
        qos: Vec<u8>,
    ) -> Packet<VariableHeaderPacketIdentifier, SubscribePayload>;
    fn unsubscribe(
        &self,
        packet_identifier: u16,
        topic_names: Vec<String>,
    ) -> Packet<VariableHeaderPacketIdentifier, UnsubscribePayload>;
}
impl<T, P> ClientPacket for Packet<T, P> {
    /// Creates a Connect packet with credentials
    fn connect_with_credentials(
        &self,
        client_identifier: String,
        user_name: String,
        password: String,
        clean_session: bool,
        will_topic: String,
        will_message: String,
    ) -> Packet<VariableHeader, Payload> {
        let payload = payload_packet::Payload {
            client_identifier: client_identifier.clone(),
            user_name,
            password,
            will_topic: will_topic.clone(),
            will_message: will_message.clone(),
            ..Default::default()
        };
        let mut packet = self.connect(client_identifier, clean_session, will_topic, will_message);
        packet.payload = payload;
        packet
    }

    /// Creates a Connect packet
    fn connect(
        &self,
        client_identifier: String,
        clean_session: bool,
        will_topic: String,
        will_message: String,
    ) -> Packet<VariableHeader, Payload> {
        let header = Header {
            control_type: control_type::CONNECT,    // 0x10
            control_flags: control_flags::RESERVED, // 0x00
            remaining_length_0: vec![0], // what remaining lenght is? (how it is calculated)
        };
        let protocol_name = [0x00, 0x04, b'M', b'Q', b'T', b'T'].to_vec();
        let variable_header: VariableHeader = VariableHeader {
            protocol_name,
            protocol_level: 0x04,
            connect_flags: if clean_session {
                connect_flags::CLEAN_SESSION
            } else {
                connect_flags::RESERVED
            },
            keep_alive: 0x00,
        };
        let payload = Payload {
            client_identifier,
            will_topic,
            will_message,
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
        let remaining_length =
            (packet.variable_header.value().len() + packet.payload.value().len()) as u32;
        // println!("calculated packet remaining length: {}", remaining_length);
        packet.header.set_remaining_length(remaining_length);
        // println!(" decoded remaining length {:?}", packet.header.decode_remaining_length());
        packet
    }

    /// Creates a Disconnect packet
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

    /// Creates a PingReq packet
    fn pingreq(&self) -> Packet<VariableHeader, Payload> {
        let header = Header {
            control_type: control_type::PINGREQ,
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

    /// Creates a PubAck packet
    fn puback(&self, packet_identifier: u16) -> Packet<VariableHeaderPacketIdentifier, Payload> {
        let header = Header {
            control_type: control_type::PUBACK,
            control_flags: control_flags::RESERVED,
            remaining_length_0: vec![2],
        };
        let variable_header = VariableHeaderPacketIdentifier { packet_identifier };
        // building the struct packet
        Packet::<VariableHeaderPacketIdentifier, Payload> {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: false,
            payload: Payload::default(),
        }
    }

    /// Creates a Publish packet
    fn publish(
        &self,
        dup: u8,
        qos: u8,
        retain: u8,
        packet_identifier: u16,
        topic_name: String,
        message: String,
    ) -> Packet<VariableHeaderPublish, PublishPayload> {
        let header = Header {
            control_type: control_type::PUBLISH,
            control_flags: (dup << 4) as u8 | (qos << 1) as u8 | retain as u8,
            remaining_length_0: vec![0],
        };
        let variable_header = VariableHeaderPublish {
            topic_name: topic_name.as_bytes().to_vec(),
            packet_identifier,
        };

        let payload = PublishPayload { message };
        // building the struct packet
        let mut packet = Packet {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: true,
            payload,
        };
        let remaining_length =
            (packet.variable_header.value().len() + packet.payload.value().len()) as u32;
        packet.header.set_remaining_length(remaining_length);
        packet
    }

    /// Creates a Subscribe packet
    fn subscribe(
        &self,
        packet_identifier: u16,
        topic_names: Vec<String>,
        qos: Vec<u8>,
    ) -> Packet<VariableHeaderPacketIdentifier, SubscribePayload> {
        let mut header = Header {
            control_type: control_type::SUBSCRIBE,
            control_flags: control_flags::RESERVED,
            remaining_length_0: vec![0],
        };
        let variable_header = VariableHeaderPacketIdentifier { packet_identifier };
        let payload = SubscribePayload {
            topic_filter: topic_names,
            qos,
        };
        header.set_remaining_length((variable_header.value().len() + payload.value().len()) as u32);
        // building the struct packet
        Packet::<VariableHeaderPacketIdentifier, SubscribePayload> {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: true,
            payload,
        }
    }

    /// Creates a Unsubscribe packet
    fn unsubscribe(
        &self,
        packet_identifier: u16,
        topic_names: Vec<String>,
    ) -> Packet<VariableHeaderPacketIdentifier, UnsubscribePayload> {
        let mut header = Header {
            control_type: control_type::UNSUBSCRIBE,
            control_flags: control_flags::QOS0,
            remaining_length_0: vec![0],
        };
        let variable_header = VariableHeaderPacketIdentifier { packet_identifier };
        let payload = UnsubscribePayload {
            topic_filter: topic_names,
        };
        header.set_remaining_length((variable_header.value().len() + payload.value().len()) as u32);
        // building the struct packet
        Packet::<VariableHeaderPacketIdentifier, UnsubscribePayload> {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: true,
            payload,
        }
    }
}

pub trait ServerPacket {
    fn connack(
        &self,
        connect_ack_flags: u8,
        connect_return: u8,
    ) -> Packet<VariableHeaderConnack, Payload>;
    fn pingresp(&self) -> Packet<VariableHeader, Payload>;
    fn suback(
        &self,
        packet_identifier: u16,
        qos: Vec<u8>,
    ) -> Packet<VariableHeaderPacketIdentifier, SubackPayload>;
    fn unsuback(&self, packet_identifier: u16) -> Packet<VariableHeaderPacketIdentifier, Payload>;
}

impl<T, P> ServerPacket for Packet<T, P> {
    /// Creates a Connack packet
    fn connack(
        &self,
        connect_ack_flags: u8,
        connect_return: u8,
    ) -> Packet<VariableHeaderConnack, Payload> {
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
        let remaining_length = (variable_header.value().len()) as u32;
        header.set_remaining_length(remaining_length);

        Packet {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: false,
            payload,
        }
    }

    /// Creates a Pingresp packet
    fn pingresp(&self) -> Packet<VariableHeader, Payload> {
        let header = Header {
            control_type: control_type::PINGRESP,
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

    /// Creates a Suback packet
    fn suback(
        &self,
        packet_identifier: u16,
        qos: Vec<u8>,
    ) -> Packet<VariableHeaderPacketIdentifier, SubackPayload> {
        let mut header = Header {
            control_type: control_type::SUBACK,
            control_flags: control_flags::RESERVED,
            remaining_length_0: vec![0],
        };
        let variable_header = VariableHeaderPacketIdentifier { packet_identifier };

        let payload = SubackPayload { qos };
        header.set_remaining_length((variable_header.value().len() + payload.value().len()) as u32);
        // building the struct packet
        Packet::<VariableHeaderPacketIdentifier, SubackPayload> {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: true,
            payload,
        }
    }

    /// Creates a Unsuback packet
    fn unsuback(&self, packet_identifier: u16) -> Packet<VariableHeaderPacketIdentifier, Payload> {
        let mut header = Header {
            control_type: control_type::UNSUBACK,
            control_flags: control_flags::RESERVED,
            remaining_length_0: vec![0],
        };
        let variable_header = VariableHeaderPacketIdentifier { packet_identifier };
        header.set_remaining_length(variable_header.value().len() as u32);
        // building the struct packet
        Packet {
            header,
            has_variable_header: true,
            variable_header,
            has_payload: false,
            payload: Payload::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod packets {
        use crate::mqtt_packet_service::variable_header_packet::{
            connect_ack_flags, connect_return,
        };

        use super::*;
        use payload_packet::suback_return_codes;

        #[test]
        fn check_unsuback_packet() {
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.unsuback(0x1234);
            let value = packet.value();
            assert_eq!(value.len(), 4);
            let unvalue = Packet::<VariableHeaderPacketIdentifier, Payload>::unvalue(value);
            assert_eq!(unvalue.header.control_type, control_type::UNSUBACK);
            assert_eq!(unvalue.header.control_flags, control_flags::RESERVED);
            assert_eq!(unvalue.header.remaining_length_0[0], 2);
            assert_eq!(unvalue.variable_header.packet_identifier, 0x1234);
        }

        #[test]
        fn check_unsubscribe_packet() {
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.unsubscribe(1, vec!["topic1".to_string(), "topic2".to_string()]);
            let value = packet.value();
            assert_eq!(value.len(), 20);
            let unvalue =
                Packet::<VariableHeaderPacketIdentifier, UnsubscribePayload>::unvalue(value);

            assert_eq!(unvalue.header.control_type, control_type::UNSUBSCRIBE);
            assert_eq!(unvalue.header.control_flags, control_flags::QOS0);
            assert_eq!(unvalue.header.remaining_length_0[0], 18);
            assert_eq!(unvalue.variable_header.packet_identifier, 1);
            assert_eq!(unvalue.payload.topic_filter[0], "topic1".to_string());
            assert_eq!(unvalue.payload.topic_filter[1], "topic2".to_string());
        }

        #[test]
        fn check_suback_packet() {
            let qos_stub = vec![
                suback_return_codes::SUCCESS_QOS0,
                suback_return_codes::SUCCESS_QOS1,
                suback_return_codes::FAILURE,
            ];
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.suback(10, qos_stub.clone());
            assert_eq!(packet.header.control_type, control_type::SUBACK);
            assert_eq!(packet.header.control_flags, control_flags::RESERVED);
            assert_eq!(packet.header.remaining_length_0, vec![5]);
            assert_eq!(packet.variable_header.packet_identifier, 10);
            assert_eq!(packet.payload.qos, qos_stub);
            let value = packet.value();
            let unvalue = Packet::<VariableHeaderPacketIdentifier, SubackPayload>::unvalue(value);
            assert_eq!(unvalue.header.control_type, control_type::SUBACK);
            assert_eq!(unvalue.header.control_flags, control_flags::RESERVED);
            assert_eq!(unvalue.header.remaining_length_0, vec![5]);
            assert_eq!(unvalue.variable_header.packet_identifier, 10);
            assert_eq!(unvalue.payload.qos, qos_stub);
        }

        #[test]
        fn check_subscribe_packet() {
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.subscribe(
                10,
                vec![String::from("topic1"), String::from("topic2")],
                vec![0, 1],
            );
            let remaining_len: u8 = 20;
            assert_eq!(packet.header.control_type, control_type::SUBSCRIBE);
            assert_eq!(packet.header.control_flags, control_flags::RESERVED);
            assert_eq!(packet.header.remaining_length_0, [remaining_len]);
            assert_eq!(packet.variable_header.packet_identifier, 10);
            assert_eq!(
                packet.payload.topic_filter,
                vec![String::from("topic1"), String::from("topic2")]
            );
            assert_eq!(packet.payload.qos, vec![0, 1]);

            let value = packet.value();
            // println!("{:?}", value);
            assert_eq!(value[0], control_type::SUBSCRIBE);
            assert_eq!(value[1], remaining_len);

            let unvalue =
                Packet::<VariableHeaderPacketIdentifier, SubscribePayload>::unvalue(value);
            // println!("{:?}", unvalue);
            assert_eq!(unvalue.header.control_type, control_type::SUBSCRIBE);
            assert_eq!(unvalue.header.control_flags, control_flags::RESERVED);
            assert_eq!(unvalue.header.remaining_length_0, [remaining_len]);
            assert_eq!(unvalue.variable_header.packet_identifier, 10);
            assert_eq!(
                unvalue.payload.topic_filter,
                vec![String::from("topic1"), String::from("topic2")]
            );
            assert_eq!(unvalue.payload.qos, vec![0, 1]);
        }

        #[test]
        fn check_pingresp_packet() {
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.pingresp();
            assert_eq!(packet.header.control_type, control_type::PINGRESP);
            assert_eq!(packet.header.control_flags, control_flags::RESERVED);
            assert_eq!(packet.header.remaining_length_0, vec![0]);
            assert_eq!(packet.has_variable_header, false);
            assert_eq!(packet.has_payload, false);
            let value = packet.value();
            // println!("value: {:?}", value);
            assert_eq!(value.len(), 2);
            assert_eq!(value[0], control_type::PINGRESP + control_flags::RESERVED);
            assert_eq!(value[1], 0);

            let unvalue = Packet::<VariableHeader, Payload>::unvalue(value);
            // println!("unvalue: {:?}", unvalue);
            assert_eq!(unvalue.header.control_type, control_type::PINGRESP);
            assert_eq!(unvalue.header.control_flags, control_flags::RESERVED);
            assert_eq!(unvalue.header.remaining_length_0, vec![0]);
        }

        #[test]
        fn check_is_mqtt_packet() {
            let test1: Vec<u8> = vec![];
            let test2: Vec<u8> = vec![0x00];
            let test3: Vec<u8> = vec![0x10, 0x00, 0x00];
            assert_eq!(
                Packet::<VariableHeader, Payload>::is_mqtt_packet(&test1),
                false
            );
            assert_eq!(
                Packet::<VariableHeader, Payload>::is_mqtt_packet(&test2),
                false
            );
            assert_eq!(
                Packet::<VariableHeader, Payload>::is_mqtt_packet(&test3),
                true
            );
        }

        #[test]
        fn check_packet_publishack() {
            let packet = Packet::<VariableHeaderPacketIdentifier, Payload>::new();
            let packet = packet.puback(1);
            assert_eq!(packet.header.control_type, control_type::PUBACK);
            assert_eq!(packet.header.control_flags, control_flags::RESERVED);
            assert_eq!(packet.header.remaining_length_0, vec![2]);
            assert_eq!(packet.variable_header.packet_identifier, 1);
            let value = packet.value();
            assert_eq!(vec![64, 2, 0, 1], value);
            let unvalue = Packet::<VariableHeaderPacketIdentifier, Payload>::unvalue(value);
            assert_eq!(
                packet.header.remaining_length_0,
                unvalue.header.remaining_length_0
            );
            assert_eq!(packet.header.control_type, unvalue.header.control_type);
            assert_eq!(packet.header.control_flags, unvalue.header.control_flags);
            assert_eq!(
                packet.variable_header.packet_identifier,
                unvalue.variable_header.packet_identifier
            );
        }

        #[test]
        fn test_packet_remaining_len() {
            // let connect_head_stub = vec![0x10, 18, 0, 4, 77, 81, 84, 84, 4, 2, 0, 0];
            let client_identifier = String::from("testId");
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.connect(client_identifier, true,"".to_string(), "".to_string());
            let mut readed: usize = 0;
            let value = packet.value();
            let remaining_len = Packet::<VariableHeader, Payload>::get_packet_length(
                &value[1..value.len()].to_vec(),
                &mut readed,
            );
            assert_eq!(remaining_len, 26);
            assert_eq!(readed, 1);
        }

        #[test]
        fn test_unvalue_variableheader_payload() {
            let client_identifier = String::from("testId");
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.connect(client_identifier, true, "".to_string(), "".to_string());
            let value = packet.value();
            // println!("packet bytes: {:?}", value);
            let unvalued_packet = Packet::<VariableHeader, Payload>::unvalue(value.clone());
            // println!("unvalue {:?}", unvalued_packet);
            assert_eq!(value, unvalued_packet.value());
        }

        #[test]
        fn check_connect_packet() {
            let connect_head_stub = vec![0x10, 26, 0, 4, 77, 81, 84, 84, 4, 2, 0, 0];
            let client_identifier = String::from("testId");
            let connect_stub: Vec<u8> = connect_head_stub
                .iter()
                .copied()
                .chain(
                    (client_identifier.len() as u16)
                        .to_be_bytes()
                        .iter()
                        .copied()
                        .chain(
                            client_identifier
                                .as_bytes()
                                .iter()
                                .copied()
                                .chain(vec![0 as u8; 8].iter().copied()),
                        ),
                )
                .collect();
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.connect(client_identifier, true, "".to_string(), "".to_string());
            let value = packet.value();
            // println!("value connect: {:?}", value);
            // println!("connect stub: {:?}", connect_stub);
            assert_eq!(value.len(), connect_stub.len());
            assert!(connect_stub.eq(&value));

            let unvalued_packet = Packet::<VariableHeader, Payload>::unvalue(value);
            // println!("unvalue {:?}", unvalued_packet);
            assert_eq!(unvalued_packet.value().len(), connect_stub.len());
            assert_eq!(unvalued_packet.header.control_type, control_type::CONNECT);
            assert_eq!(
                unvalued_packet.header.control_flags,
                control_flags::RESERVED
            );
            assert_eq!(unvalued_packet.header.remaining_length_0, vec![26]);
            assert_eq!(
                unvalued_packet.variable_header.protocol_name,
                vec![0, 4, 77, 81, 84, 84]
            );
            assert_eq!(unvalued_packet.variable_header.protocol_level, 4);
            assert_eq!(
                unvalued_packet.variable_header.connect_flags,
                connect_flags::CLEAN_SESSION
            );
            assert_eq!(unvalued_packet.variable_header.keep_alive, 0);
        }

        #[test]
        fn check_connack_packet() {
            let header = vec![0x20, 0x02];
            let mut variable_header = Vec::with_capacity(2);
            variable_header.push(connect_ack_flags::SESSION_PRESENT);
            variable_header.push(connect_return::ACCEPTED);
            let connack_head_stub: Vec<u8> = header
                .iter()
                .copied()
                .chain(variable_header.iter().copied())
                .collect();

            let packet = Packet::<VariableHeader, Payload>::new();
            let packet =
                packet.connack(connect_ack_flags::SESSION_PRESENT, connect_return::ACCEPTED);
            let value = packet.value();
            // println!("value connack: {:?}", value);
            // println!("connack stub: {:?}", connack_head_stub);
            assert_eq!(value.len(), connack_head_stub.len());
            assert!(connack_head_stub.eq(&value));

            let unvalued_packet = Packet::<VariableHeaderConnack, Payload>::unvalue(value);
            assert_eq!(unvalued_packet.header.control_type, control_type::CONNACK);
            assert_eq!(
                unvalued_packet.header.control_flags,
                control_flags::RESERVED
            );
            assert_eq!(unvalued_packet.header.remaining_length_0, vec![2]);
            assert_eq!(
                unvalued_packet.variable_header.acknoledge_flags,
                connect_ack_flags::SESSION_PRESENT
            );
            assert_eq!(
                unvalued_packet.variable_header.return_code,
                connect_return::ACCEPTED
            );
        }

        #[test]
        fn check_disconnect_packet() {
            let header = vec![224, 0x00];
            let disconnect_stub: Vec<u8> = header.iter().copied().collect();
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.disconnect();
            let value = packet.value();
            // println!("value disconnect: {:?}", value);
            // println!("disconnect stub: {:?}", disconnect_stub);
            assert!(value.len() == disconnect_stub.len());
            assert!(disconnect_stub.eq(&value));

            let unvalued_packet = Packet::<VariableHeader, Payload>::unvalue(value);
            // println!("unvalue {:?}", unvalued_packet);
            assert_eq!(
                unvalued_packet.header.control_type,
                control_type::DISCONNECT
            );
            assert_eq!(
                unvalued_packet.header.control_flags,
                control_flags::RESERVED
            );
            assert_eq!(unvalued_packet.header.remaining_length_0, vec![0]);
        }

        #[test]
        fn check_pingreq_packet() {
            let header = vec![192, 0x00];
            let pingreq_stub: Vec<u8> = header.iter().copied().collect();
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.pingreq();
            let value = packet.value();
            // println!("value pingreq: {:?}", value);
            // println!("pingreq stub: {:?}", pingreq_stub);
            assert!(value.len() == pingreq_stub.len());
            assert!(pingreq_stub.eq(&value));

            let value = packet.value();
            // println!("value pingreq: {:?}", value);
            assert_eq!(value.len(), pingreq_stub.len());
            assert!(pingreq_stub.eq(&value));

            let unvalued_packet = Packet::<VariableHeader, Payload>::unvalue(value);
            // println!("unvalue {:?}", unvalued_packet);
            assert_eq!(unvalued_packet.header.control_type, control_type::PINGREQ);
            assert_eq!(
                unvalued_packet.header.control_flags,
                control_flags::RESERVED
            );
            assert_eq!(unvalued_packet.header.remaining_length_0, vec![0]);
        }

        #[test]
        fn check_publish_packet() {
            let dup = 0;
            let qos = 1;
            let retain = 0;
            let header = vec![
                (control_type::PUBLISH + (((dup << 4) as u8 | (qos << 1) as u8 | retain) as u8))
                    as u8,
                24,
            ]; // length of 24 for this example
            let topic_name = String::from("testTopic");
            let topic_name_vec = "testTopic".as_bytes().to_vec();
            let packet_identifier: u8 = 10;
            let packet_identifier_vec: Vec<u8> = vec![0, packet_identifier];
            let payload = String::from("testPayload");

            let publish_stub: Vec<u8> = header
                .iter()
                .copied()
                .chain(
                    (topic_name_vec.len() as u16)
                        .to_be_bytes()
                        .iter()
                        .copied()
                        .chain(
                            topic_name_vec.iter().copied().chain(
                                packet_identifier_vec.iter().copied().chain(
                                    (payload.len() as u16)
                                        .to_be_bytes()
                                        .iter()
                                        .copied()
                                        .chain(payload.as_bytes().iter().copied()),
                                ),
                            ),
                        ),
                )
                .collect();
            let packet = Packet::<VariableHeader, Payload>::new();
            let packet = packet.publish(
                dup,
                qos,
                retain,
                packet_identifier as u16,
                topic_name.clone(),
                payload.clone(),
            );
            let value = packet.value();
            // println!("value publish: {:?}", value);
            // println!("publish stub: {:?}", publish_stub);
            assert!(value.len() == publish_stub.len());
            assert!(publish_stub.eq(&value));
            let unvalue = Packet::<VariableHeaderPublish, PublishPayload>::unvalue(value);
            // println!("unvalue {:?}", unvalue);
            assert_eq!(unvalue.header.control_type, control_type::PUBLISH);
            assert_eq!(
                unvalue.header.control_flags,
                (((dup << 4) as u8 | (qos << 1) as u8 | retain) as u8) as u8
            );
            assert_eq!(unvalue.header.remaining_length_0, vec![24]);
            assert_eq!(
                unvalue.variable_header.topic_name,
                topic_name.clone().as_bytes().to_vec()
            );
            assert_eq!(
                unvalue.variable_header.packet_identifier,
                packet_identifier as u16
            );
            assert_eq!(unvalue.payload.message, payload.clone());
        }
    }
}
