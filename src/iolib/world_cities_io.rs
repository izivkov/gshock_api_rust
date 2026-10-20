use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

pub fn world_cities_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::WorldCities as u8]))
}

pub fn encode_and_pad_world_city(city_name: &str, city_number: u8) -> [u8; 19] {
    let mut bytes = [0u8; 19];
    bytes[0] = Protocol::WorldCities as u8;
    bytes[1] = city_number;

    let ascii_bytes = city_name.as_bytes();
    let len = ascii_bytes.len().min(17);
    bytes[2..2 + len].copy_from_slice(&ascii_bytes[..len]);
    bytes
}

pub fn parse_city_name(time_zone_name: &str) -> String {
    time_zone_name
        .split('/')
        .last()
        .unwrap_or(time_zone_name)
        .split(':')
        .last()
        .unwrap_or(time_zone_name)
        .to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_cities() {
        assert_eq!(
            world_cities_request(),
            BleAction::Write(Write::new(0x000c, [0x1f]))
        );

        let encoded = encode_and_pad_world_city("TOKYO", 1);
        assert_eq!(encoded[0], 0x1f);
        assert_eq!(encoded[1], 1);
        assert_eq!(&encoded[2..7], b"TOKYO");

        assert_eq!(parse_city_name("Europe/London"), "LONDON");
    }
}
