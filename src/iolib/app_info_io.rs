use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

pub fn app_info_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::AppInfo as u8]))
}

pub fn prepare_app_info_response(data: &[u8]) -> Option<BleAction> {
    if data.len() >= 12
        && data[0] == Protocol::AppInfo as u8
        && data[1..11] == [0xff; 10]
        && data[11] == 0x00
    {
        let mut packet = Vec::with_capacity(13);
        packet.push(Protocol::AppInfo as u8);
        packet.extend_from_slice(&[0x34, 0x88, 0xf4, 0xe5, 0xd5, 0xaf, 0xc8, 0x29, 0xe0, 0x6d]);
        packet.push(0x02);
        Some(BleAction::Write(Write::new(0x000e, packet)))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_info_request_and_response() {
        assert_eq!(
            app_info_request(),
            BleAction::Write(Write::new(0x000c, [0x22]))
        );

        let mut trigger = vec![0x22];
        trigger.extend_from_slice(&[0xff; 10]);
        trigger.push(0x00);

        let response = prepare_app_info_response(&trigger).unwrap();
        assert_eq!(
            response,
            BleAction::Write(Write::new(
                0x000e,
                vec![0x22, 0x34, 0x88, 0xf4, 0xe5, 0xd5, 0xaf, 0xc8, 0x29, 0xe0, 0x6d, 0x02]
            ))
        );
    }
}
