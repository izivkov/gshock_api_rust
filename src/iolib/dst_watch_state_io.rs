use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

pub fn dst_watch_state_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::DstWatchState as u8]))
}

pub fn set_dst_watch_state(original_data: &[u8], dst_value: u8) -> Vec<u8> {
    let mut data = original_data.to_vec();
    if data.len() > 3 {
        data[3] = dst_value;
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dst_watch_state() {
        assert_eq!(
            dst_watch_state_request(),
            BleAction::Write(Write::new(0x000c, [0x1d]))
        );

        let orig = vec![0x1d, 0x00, 0x00, 0x00];
        let res = set_dst_watch_state(&orig, 2);
        assert_eq!(res[3], 2);
    }
}
