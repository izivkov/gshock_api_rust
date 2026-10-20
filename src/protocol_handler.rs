use crate::watch_info::ProtocolKind;

pub fn extract_key(protocol: ProtocolKind, data: &[u8]) -> Option<u8> {
    if data.is_empty() {
        return None;
    }

    match protocol {
        ProtocolKind::Analogue => {
            let first_byte = data[0];
            if first_byte == 0x28 && data.len() > 4 {
                if data[1] == 0x01 {
                    Some(data[4])
                } else if data[1] == 0x00 {
                    Some(data[3])
                } else {
                    Some(0x28)
                }
            } else {
                Some(first_byte)
            }
        }
        ProtocolKind::Standard | ProtocolKind::Mip => {
            let key = data[0];
            if key == 0 { None } else { Some(key) }
        }
    }
}

pub fn unwrap_payload<'a>(protocol: ProtocolKind, key: u8, data: &'a [u8]) -> &'a [u8] {
    if data.is_empty() {
        return data;
    }

    match protocol {
        ProtocolKind::Analogue => {
            if data[0] == 0x28 && key != 0x28 {
                let skip = if data.len() > 1 && data[1] == 0x01 { 4 } else { 3 };
                if data.len() >= skip {
                    &data[skip..]
                } else {
                    data
                }
            } else {
                data
            }
        }
        ProtocolKind::Standard | ProtocolKind::Mip => data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_key_is_ignored() {
        assert_eq!(extract_key(ProtocolKind::Standard, &[0x00, 0x05, 0x00, 0x00]), None);
        assert_eq!(extract_key(ProtocolKind::Standard, &[0x11, 0x0f, 0x0f]), Some(0x11));
    }

    #[test]
    fn test_analogue_protocol_key_extraction() {
        // Wrapped 0x28 analogue packet
        let wrapped1 = [0x28, 0x01, 0x00, 0x00, 0x15, 0x40, 0x40, 7, 25];
        assert_eq!(extract_key(ProtocolKind::Analogue, &wrapped1), Some(0x15));
        let unwrapped1 = unwrap_payload(ProtocolKind::Analogue, 0x15, &wrapped1);
        assert_eq!(unwrapped1, &[0x15, 0x40, 0x40, 7, 25]);
    }
}
