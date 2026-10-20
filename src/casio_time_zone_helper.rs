#[derive(Clone, Debug, PartialEq)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

impl LatLon {
    pub const fn new(lat: f64, lon: f64) -> Self {
        Self { lat, lon }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CasioTimeZone {
    pub name: &'static str,
    pub zone_name: &'static str,
    pub dst_rules: u8,
}

impl CasioTimeZone {
    pub const fn new(name: &'static str, zone_name: &'static str, dst_rules: u8) -> Self {
        Self {
            name,
            zone_name,
            dst_rules,
        }
    }
}

pub const TIME_ZONE_TABLE: &[CasioTimeZone] = &[
    CasioTimeZone::new("BAKER ISLAND", "UTC-12", 0x00),
    CasioTimeZone::new("MARQUESAS ISLANDS", "Pacific/Marquesas", 0xDA),
    CasioTimeZone::new("POGO POGO", "Pacific/Pago_Pago", 0x00),
    CasioTimeZone::new("HONOLULU", "Pacific/Honolulu", 0x00),
    CasioTimeZone::new("ANCHORAGE", "America/Anchorage", 0x01),
    CasioTimeZone::new("LOS ANGELES", "America/Los_Angeles", 0x01),
    CasioTimeZone::new("DENVER", "America/Denver", 0x01),
    CasioTimeZone::new("CHICAGO", "America/Chicago", 0x01),
    CasioTimeZone::new("NEW YORK", "America/New_York", 0x01),
    CasioTimeZone::new("HALIFAX", "America/Halifax", 0x01),
    CasioTimeZone::new("ST.JOHN'S", "America/St_Johns", 0x01),
    CasioTimeZone::new("RIO DE JANEIRO", "America/Sao_Paulo", 0x00),
    CasioTimeZone::new("F.DE NORONHA", "America/Noronha", 0x00),
    CasioTimeZone::new("PRAIA", "Atlantic/Cape_Verde", 0x00),
    CasioTimeZone::new("UTC", "UTC", 0x00),
    CasioTimeZone::new("LONDON", "Europe/London", 0x02),
    CasioTimeZone::new("PARIS", "Europe/Paris", 0x02),
    CasioTimeZone::new("ATHENS", "Europe/Athens", 0x02),
    CasioTimeZone::new("JEDDAH", "Asia/Riyadh", 0x00),
    CasioTimeZone::new("JERUSALEM", "Asia/Jerusalem", 0x2A),
    CasioTimeZone::new("TEHRAN", "Asia/Tehran", 0x2B),
    CasioTimeZone::new("DUBAI", "Asia/Dubai", 0x00),
    CasioTimeZone::new("KABUL", "Asia/Kabul", 0x00),
    CasioTimeZone::new("KARACHI", "Asia/Karachi", 0x00),
    CasioTimeZone::new("DELHI", "Asia/Kolkata", 0x00),
    CasioTimeZone::new("KATHMANDU", "Asia/Kathmandu", 0x00),
    CasioTimeZone::new("DHAKA", "Asia/Dhaka", 0x00),
    CasioTimeZone::new("YANGON", "Asia/Yangon", 0x00),
    CasioTimeZone::new("BANGKOK", "Asia/Bangkok", 0x00),
    CasioTimeZone::new("HONG KONG", "Asia/Hong_Kong", 0x00),
    CasioTimeZone::new("PYONGYANG", "Asia/Pyongyang", 0x00),
    CasioTimeZone::new("EUCLA", "Australia/Eucla", 0x00),
    CasioTimeZone::new("TOKYO", "Asia/Tokyo", 0x00),
    CasioTimeZone::new("ADELAIDE", "Australia/Adelaide", 0x04),
    CasioTimeZone::new("SYDNEY", "Australia/Sydney", 0x04),
    CasioTimeZone::new("LORD HOWE ISLAND", "Australia/Lord_Howe", 0x12),
    CasioTimeZone::new("NOUMEA", "Pacific/Noumea", 0x00),
    CasioTimeZone::new("WELLINGTON", "Pacific/Auckland", 0x05),
    CasioTimeZone::new("CHATHAM ISLANDS", "Pacific/Chatham", 0x17),
    CasioTimeZone::new("NUKUALOFA", "Pacific/Tongatapu", 0x00),
    CasioTimeZone::new("KIRITIMATI", "Pacific/Kiritimati", 0x00),
    CasioTimeZone::new("CASABLANCA", "Africa/Casablanca", 0x0F),
    CasioTimeZone::new("BEIRUT", "Asia/Beirut", 0x0C),
    CasioTimeZone::new("NORFOLK ISLAND", "Pacific/Norfolk", 0x04),
    CasioTimeZone::new("EASTER ISLAND", "Pacific/Easter", 0x1C),
    CasioTimeZone::new("HAVANA", "America/Havana", 0x15),
    CasioTimeZone::new("SANTIAGO", "America/Santiago", 0x1B),
    CasioTimeZone::new("ASUNCION", "America/Asuncion", 0x09),
    CasioTimeZone::new("PONTA DELGADA", "Atlantic/Azores", 0x02),
];

pub fn find_time_zone(zone_name: &str) -> CasioTimeZone {
    for entry in TIME_ZONE_TABLE {
        if entry.zone_name == zone_name || entry.name == zone_name.to_uppercase() {
            return entry.clone();
        }
    }
    // Fallback
    CasioTimeZone {
        name: "UTC",
        zone_name: "UTC",
        dst_rules: 0x00,
    }
}

pub fn get_world_city_coordinates(zone_id: &str) -> (f64, f64, bool) {
    match zone_id {
        "Asia/Ho_Chi_Minh" => (10.7958, 106.7062, true),
        "Europe/Madrid" => (41.4548, 2.2502, true),
        "Asia/Shanghai" => (22.7230, 114.2611, true),
        "UTC-12" => (0.1936, -176.4769, true),
        "Pacific/Marquesas" => (-8.9167, -140.1000, true),
        "Pacific/Pago_Pago" => (-14.2781, -170.7025, true),
        "Pacific/Honolulu" => (21.3069, -157.8583, true),
        "America/Anchorage" => (61.2181, -149.9003, true),
        "America/Los_Angeles" => (34.0522, -118.2437, true),
        "America/Denver" => (39.7392, -104.9903, true),
        "America/Chicago" => (41.8781, -87.6298, true),
        "America/New_York" => (40.7128, -74.0060, true),
        "America/Halifax" => (44.6488, -63.5752, true),
        "America/St_Johns" => (47.5615, -52.7126, true),
        "America/Sao_Paulo" => (-22.9068, -43.1729, true),
        "America/Noronha" => (-3.8536, -32.4297, true),
        "Atlantic/Cape_Verde" => (14.9330, -23.5133, true),
        "UTC" => (0.0, 0.0, true),
        "Europe/London" => (51.5074, -0.1278, true),
        "Europe/Paris" => (48.8566, 2.3522, true),
        "Europe/Athens" => (37.9838, 23.7275, true),
        "Asia/Riyadh" => (21.4858, 39.1925, true),
        "Asia/Jerusalem" => (31.7683, 35.2137, true),
        "Asia/Tehran" => (35.6892, 51.3890, true),
        "Asia/Dubai" => (25.2048, 55.2708, true),
        "Asia/Kabul" => (34.5553, 69.2075, true),
        "Asia/Karachi" => (24.8607, 67.0011, true),
        "Asia/Kolkata" => (28.6139, 77.2090, true),
        "Asia/Kathmandu" => (27.7172, 85.3240, true),
        "Asia/Dhaka" => (23.8103, 90.4125, true),
        "Asia/Yangon" => (16.8661, 96.1951, true),
        "Asia/Bangkok" => (13.7563, 100.5018, true),
        "Asia/Hong_Kong" => (22.3193, 114.1694, true),
        "Asia/Pyongyang" => (39.0392, 125.7625, true),
        "Australia/Eucla" => (-31.6784, 128.8869, true),
        "Asia/Tokyo" => (35.6762, 139.6503, true),
        "Australia/Adelaide" => (-34.9285, 138.6007, true),
        "Australia/Sydney" => (-33.8688, 151.2093, true),
        "Australia/Lord_Howe" => (-31.5553, 159.0821, true),
        "Pacific/Noumea" => (-22.2758, 166.4581, true),
        "Pacific/Auckland" => (-41.2865, 174.7762, true),
        "Pacific/Chatham" => (-43.9500, -176.5500, true),
        "Pacific/Tongatapu" => (-21.1789, -175.1982, true),
        "Pacific/Kiritimati" => (1.8721, -157.4278, true),
        "Africa/Casablanca" => (33.5731, -7.5898, true),
        "Asia/Beirut" => (33.8938, 35.5018, true),
        "Pacific/Norfolk" => (-29.0408, 167.9547, true),
        "Pacific/Easter" => (-27.1127, -109.3497, true),
        "America/Havana" => (23.1136, -82.3666, true),
        "America/Santiago" => (-33.4489, -70.6693, true),
        "America/Asuncion" => (-25.2637, -57.5759, true),
        "Atlantic/Azores" => (37.7412, -25.6756, true),
        _ => (0.0, 0.0, false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_casio_time_zone_helper() {
        let (lat, lon, exact) = get_world_city_coordinates("Europe/Madrid");
        assert!(exact);
        assert!((lat - 41.4548).abs() < 0.0001);
        assert!((lon - 2.2502).abs() < 0.0001);

        let tz = find_time_zone("Europe/London");
        assert_eq!(tz.name, "LONDON");
        assert_eq!(tz.zone_name, "Europe/London");
    }
}
