use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

pub fn time_adjustment_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::SettingForBle as u8]))
}

pub fn decode_time_adjustment(data: &[u8]) -> Option<(bool, u8)> {
    if data.len() < 14 {
        return None;
    }
    let time_adjusted = data[12] == 0x00;
    let minutes_after_hour = data[13];
    Some((time_adjusted, minutes_after_hour))
}

pub fn encode_time_adjustment(
    original_data: &[u8],
    time_adjustment: bool,
    minutes_after_hour: u8,
) -> Vec<u8> {
    let mut data = original_data.to_vec();
    if data.len() >= 14 {
        data[12] = if !time_adjustment { 0x80 } else { 0x00 };
        data[13] = minutes_after_hour;
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_adjustment() {
        assert_eq!(
            time_adjustment_request(),
            BleAction::Write(Write::new(0x000c, [0x11]))
        );

        let mut orig = vec![0u8; 15];
        orig[12] = 0x80;
        orig[13] = 10;

        let encoded = encode_time_adjustment(&orig, true, 25);
        let (adjusted, minutes) = decode_time_adjustment(&encoded).unwrap();
        assert!(adjusted);
        assert_eq!(minutes, 25);
    }
}
