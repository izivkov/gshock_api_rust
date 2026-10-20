use crate::iolib::actions::{BleAction, Write};
use crate::iolib::world_cities_io::world_cities_request;

pub fn home_time_request(slot: u8, is_mtg_b3000: bool) -> BleAction {
    if is_mtg_b3000 {
        BleAction::Write(Write::new(0x000c, [0x24, slot]))
    } else {
        world_cities_request()
    }
}

pub fn parse_home_city(data: &[u8]) -> String {
    if data.len() <= 2 {
        return String::new();
    }
    let s = String::from_utf8_lossy(&data[2..]);
    s.trim_matches('\0').trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_home_time() {
        assert_eq!(
            home_time_request(0, true),
            BleAction::Write(Write::new(0x000c, [0x24, 0]))
        );

        let data = [0x24, 0x00, b'L', b'O', b'N', b'D', b'O', b'N', 0x00];
        assert_eq!(parse_home_city(&data), "LONDON");
    }
}
