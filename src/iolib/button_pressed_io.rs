use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WatchButton {
    UpperLeft = 1,
    LowerLeft = 2,
    UpperRight = 3,
    LowerRight = 4,
    NoButton = 5,
    Invalid = 6,
    Find = 7,
}

/// Decodes a BLE-feature notification into a watch-button event.
pub fn decode_button_pressed(data: &[u8]) -> WatchButton {
    if data.len() < 19 || data[0] != Protocol::BleFeatures as u8 {
        return WatchButton::Invalid;
    }
    match data[8] {
        0 | 1 => WatchButton::LowerLeft,
        2 => WatchButton::Find,
        3 => WatchButton::NoButton,
        4 => WatchButton::LowerRight,
        _ => WatchButton::LowerRight,
    }
}

/// Produces the request that asks the watch for button information.
pub fn button_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::BleFeatures as u8]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_python_left_press_fixture() {
        let payload = [
            0x10, 0x17, 0x62, 0x07, 0x38, 0x85, 0xcd, 0x7f, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(decode_button_pressed(&payload), WatchButton::LowerLeft);
    }

    #[test]
    fn rejects_invalid_packets() {
        assert_eq!(decode_button_pressed(&[]), WatchButton::Invalid);
        assert_eq!(decode_button_pressed(&[0x09; 19]), WatchButton::Invalid);
    }
}
