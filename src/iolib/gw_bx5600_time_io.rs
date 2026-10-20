pub const CITY_RECORD_FLAG: u8 = 0x01;

pub fn city_record(slot_index: u8, lat: f64, lon: f64, trailing: u8) -> [u8; 22] {
    let mut record = [0u8; 22];
    record[0] = 0x14;
    record[1] = 0x00;
    record[2] = 0x24;
    record[3] = slot_index;
    record[4] = CITY_RECORD_FLAG;
    record[5..13].copy_from_slice(&lat.to_be_bytes());
    record[13..21].copy_from_slice(&lon.to_be_bytes());
    record[21] = trailing;
    record
}

pub fn build_world_city_records(lat: f64, lon: f64, is_dst: bool) -> [u8; 66] {
    let mut records = [0u8; 66];
    let trailing = if is_dst { 1 } else { 0 };

    let home = city_record(0, lat, lon, trailing);
    let empty1 = city_record(1, 0.0, 0.0, 0);
    let empty2 = city_record(2, 0.0, 0.0, 0);

    records[..22].copy_from_slice(&home);
    records[22..44].copy_from_slice(&empty1);
    records[44..66].copy_from_slice(&empty2);
    records
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gw_bx5600_city_records() {
        let records = build_world_city_records(41.4548, 2.2502, true);
        assert_eq!(records.len(), 66);
        assert_eq!(records[0], 0x14);
        assert_eq!(records[1], 0x00);
        assert_eq!(records[2], 0x24);
        assert_eq!(records[3], 0x00); // Slot 0
        assert_eq!(records[4], 0x01); // Flag
    }
}
