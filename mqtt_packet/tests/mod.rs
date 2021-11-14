
#[cfg(test)]
mod tests {
    use super::*;

    mod mqtt_packet_test {
        use crate::mqtt_packet::variable_header_packet::{connect_ack_flags, connect_return};
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
