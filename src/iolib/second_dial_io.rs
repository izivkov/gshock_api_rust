use crate::iolib::actions::{BleAction, Write};

pub const RESET_SEQUENCE_START: [u8; 3] = [0x21, 0x00, 0x01];
pub const RESET_SEQUENCE_END: [u8; 3] = [0x21, 0x01, 0x01];

pub fn reset_sequence_start() -> BleAction {
    BleAction::Write(Write::new(0x000e, RESET_SEQUENCE_START))
}

pub fn reset_sequence_end() -> BleAction {
    BleAction::Write(Write::new(0x000e, RESET_SEQUENCE_END))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_second_dial() {
        assert_eq!(
            reset_sequence_start(),
            BleAction::Write(Write::new(0x000e, [0x21, 0x00, 0x01]))
        );
        assert_eq!(
            reset_sequence_end(),
            BleAction::Write(Write::new(0x000e, [0x21, 0x01, 0x01]))
        );
    }
}
