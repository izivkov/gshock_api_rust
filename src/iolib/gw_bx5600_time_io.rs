use crate::iolib::time_io::DateTimeParts;

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

/// Step 1 Request Payload: "051D001D00240024012402"
pub fn encode_step1_request() -> Vec<u8> {
    vec![0x05, 0x1d, 0x00, 0x1d, 0x00, 0x24, 0x00, 0x24, 0x01, 0x24, 0x02]
}

/// Step 1 Write-back: Change byte 0 from 0x05 to 0x02
pub fn encode_step1_writeback(notification_data: &[u8]) -> Vec<u8> {
    let mut data = notification_data.to_vec();
    if !data.is_empty() {
        data[0] = 0x02;
    }
    data
}

/// Step 2 Request Payload: "031E001E00..."
pub fn encode_step2_request(world_cities_count: u8) -> Vec<u8> {
    let mut req = vec![0x03];
    let blocks = (world_cities_count + 1) / 2;
    for _ in 0..blocks {
        req.extend_from_slice(&[0x1e, 0x00]);
    }
    req
}

/// Step 2 Write-back: Change byte 0 from 0x03 to 0x06 + append 66-byte city records
pub fn encode_step2_writeback(
    notification_data: &[u8],
    lat: f64,
    lon: f64,
    is_dst: bool,
) -> Vec<u8> {
    let mut data = notification_data.to_vec();
    if !data.is_empty() {
        data[0] = 0x06;
    }
    let city_records = build_world_city_records(lat, lon, is_dst);
    data.extend_from_slice(&city_records);
    data
}

/// Step 3 Request Payload: "061F001F061F011F07..."
pub fn encode_step3_request(world_cities_count: u8) -> Vec<u8> {
    let mut req = vec![0x06];
    for i in 0..world_cities_count {
        let idx = (i / 2) + if i % 2 != 0 { 6 } else { 0 };
        req.extend_from_slice(&[0x1f, idx]);
    }
    req
}

/// Step 3 Write-back: Echo unmodified notification data
pub fn encode_step3_writeback(notification_data: &[u8]) -> Vec<u8> {
    notification_data.to_vec()
}

/// Step 4 Time Command: 11-byte time payload to ALL_FEATURES (0x000E)
pub fn encode_step4_time_command(time: DateTimeParts) -> [u8; 11] {
    let casio_dow = if time.weekday == 6 { 7 } else { time.weekday + 1 };
    let sub_second = ((u64::from(time.nanosecond) * 256) / 1_000_000_000) as u8;
    [
        0x09,
        (time.year & 0xff) as u8,
        ((time.year >> 8) & 0xff) as u8,
        time.month,
        time.day,
        time.hour,
        time.minute,
        time.second,
        casio_dow,
        sub_second,
        0x01,
    ]
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

    #[test]
    fn test_gw_bx5600_step_encoders() {
        assert_eq!(encode_step1_request(), vec![0x05, 0x1d, 0x00, 0x1d, 0x00, 0x24, 0x00, 0x24, 0x01, 0x24, 0x02]);

        let notif1 = vec![0x05, 0x10, 0x20];
        assert_eq!(encode_step1_writeback(&notif1)[0], 0x02);

        let notif2 = vec![0x03, 0x10, 0x20];
        let wb2 = encode_step2_writeback(&notif2, 41.45, 2.25, true);
        assert_eq!(wb2[0], 0x06);
        assert_eq!(wb2.len(), 3 + 66);

        let dt = DateTimeParts {
            year: 2026,
            month: 5,
            day: 16,
            hour: 23,
            minute: 30,
            second: 0,
            weekday: 5,
            nanosecond: 0,
        };
        let cmd = encode_step4_time_command(dt);
        assert_eq!(cmd[0], 0x09);
        assert_eq!(cmd[1], 0xea);
        assert_eq!(cmd[2], 0x07);
    }
}
