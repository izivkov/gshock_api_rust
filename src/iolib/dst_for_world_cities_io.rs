use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

pub fn dst_world_cities_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::DstSetting as u8]))
}

pub fn set_dst_for_world_city(original_data: &[u8], offset: i32, dst_offset: i32, dst_rules: u8) -> Vec<u8> {
    let mut data = original_data.to_vec();
    if data.len() > 6 {
        data[4] = (offset & 0xff) as u8;
        data[5] = (dst_offset & 0xff) as u8;
        data[6] = dst_rules;
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dst_world_cities() {
        assert_eq!(
            dst_world_cities_request(),
            BleAction::Write(Write::new(0x000c, [0x1e]))
        );

        let orig = vec![0x1e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let res = set_dst_for_world_city(&orig, 4, 1, 2);
        assert_eq!(res[4], 4);
        assert_eq!(res[5], 1);
        assert_eq!(res[6], 2);
    }
}
