use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

/// Produces the two requests required to read standard watch alarms.
pub fn alarm_request() -> Vec<BleAction> {
    vec![
        BleAction::Write(Write::new(0x000c, [Protocol::SettingForAlm as u8])),
        BleAction::Write(Write::new(0x000c, [Protocol::SettingForAlm2 as u8])),
    ]
}

/// Produces the single alarm request used by MTG-B3000 watches.
pub fn alarm_request_mtg_b3000() -> Vec<BleAction> {
    vec![BleAction::Write(Write::new(
        0x000c,
        [Protocol::SettingForAlm as u8],
    ))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_alarm_request_matches_python_reference() {
        assert_eq!(
            alarm_request(),
            vec![
                BleAction::Write(Write::new(0x000c, [0x15])),
                BleAction::Write(Write::new(0x000c, [0x16])),
            ]
        );
    }

    #[test]
    fn mtg_b3000_uses_one_alarm_request() {
        assert_eq!(
            alarm_request_mtg_b3000(),
            vec![BleAction::Write(Write::new(0x000c, [0x15]))]
        );
    }
}
