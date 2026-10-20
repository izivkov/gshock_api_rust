use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

pub fn watch_name_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::WatchName as u8]))
}

pub fn decode_watch_name(data: &[u8]) -> String {
    if data.is_empty() || data[0] != Protocol::WatchName as u8 {
        return String::new();
    }
    let s = String::from_utf8_lossy(&data[1..]);
    s.trim_matches('\0').trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_name_request_and_decode() {
        assert_eq!(
            watch_name_request(),
            BleAction::Write(Write::new(0x000c, [0x23]))
        );

        let mut payload = vec![0x23];
        payload.extend_from_slice(b"G-SHOCK\0");
        assert_eq!(decode_watch_name(&payload), "G-SHOCK");
    }
}
