use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

/// Encodes a normal timer-set payload: command, hours, minutes, seconds, 0, 0.
pub fn encode_timer(seconds: u32) -> [u8; 6] {
    let hours = (seconds / 3600) as u8;
    let minutes = ((seconds % 3600) / 60) as u8;
    let secs = (seconds % 60) as u8;
    [Protocol::Timer as u8, hours, minutes, secs, 0, 0]
}

/// Encodes the MTG-B3000's 15-byte timer-set payload.
pub fn encode_timer_mtg_b3000(seconds: u32) -> [u8; 15] {
    let normal = encode_timer(seconds);
    let mut result = [0; 15];
    result[..4].copy_from_slice(&normal[..4]);
    result
}

/// Decodes a timer notification. Invalid or truncated data yields zero, matching Python.
pub fn decode_timer(data: &[u8]) -> u32 {
    if data.len() < 4 {
        return 0;
    }
    u32::from(data[1]) * 3600 + u32::from(data[2]) * 60 + u32::from(data[3])
}

/// Produces the read request for a timer.
pub fn timer_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::Timer as u8]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_decodes_timer() {
        assert_eq!(encode_timer(3665), [0x18, 1, 1, 5, 0, 0]);
        assert_eq!(decode_timer(&encode_timer(3665)), 3665);
        assert_eq!(decode_timer(&[0x18, 1, 1]), 0);
    }

    #[test]
    fn encodes_mtg_b3000_timer() {
        let encoded = encode_timer_mtg_b3000(600);
        assert_eq!(encoded.len(), 15);
        assert_eq!(&encoded[..4], &[0x18, 0, 10, 0]);
        assert!(encoded[4..].iter().all(|byte| *byte == 0));
    }

    #[test]
    fn timer_request_matches_python_handle() {
        assert_eq!(
            timer_request(),
            BleAction::Write(Write::new(0x000c, [0x18]))
        );
    }
}
