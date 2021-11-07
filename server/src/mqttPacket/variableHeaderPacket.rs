#[cfg(test)]
mod tests {
    use super::*;
    
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
}