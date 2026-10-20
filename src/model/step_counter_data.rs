pub const PACKET_HEADER_MARKER: u8 = 0x26;
pub const SENTINEL_BUCKET_VALUE: u16 = 0xFFFE;
pub const SENTINEL_DAILY_VALUE: u32 = 0xFFFFFFFF - 1; // 0xFFFFFFFE

pub fn decode_bcd(byte: u8) -> Result<u8, &'static str> {
    let high = byte / 16;
    let low = byte % 16;
    if high > 9 || low > 9 {
        return Err("invalid BCD byte");
    }
    Ok(high * 10 + low)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepCounterData {
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day_of_month: Option<u8>,
    pub day_of_week: Option<u8>,
    pub hourly_steps: Vec<Option<u32>>,
    pub daily_history: Vec<Option<u32>>,
    pub daily_distances: Vec<Option<u32>>,
    pub current_day_steps: Option<u32>,
    pub distance_meters: Option<u32>,
    pub pending_distance_meters: Option<u32>,
    pub raw: Option<Vec<u8>>,
    pub warnings: Vec<String>,
}

impl StepCounterData {
    pub fn unavailable() -> Self {
        Self {
            year: None,
            month: None,
            day_of_month: None,
            day_of_week: None,
            hourly_steps: Vec::new(),
            daily_history: Vec::new(),
            daily_distances: Vec::new(),
            current_day_steps: None,
            distance_meters: None,
            pending_distance_meters: None,
            raw: Some(Vec::new()),
            warnings: vec!["step counter unavailable".to_string()],
        }
    }

    pub fn parse(payload: &[u8]) -> Option<Self> {
        if payload.is_empty() || payload[0] != PACKET_HEADER_MARKER {
            return None;
        }

        let mut warnings = Vec::new();
        let mut year = None;
        let mut month = None;
        let mut day_of_month = None;

        if payload.len() >= 6 {
            let y_res = decode_bcd(payload[0]);
            let m_res = decode_bcd(payload[1]);
            let d_res = decode_bcd(payload[2]);

            match (y_res, m_res, d_res) {
                (Ok(y), Ok(m), Ok(d)) => {
                    year = Some(2000 + u16::from(y));
                    month = Some(m);
                    day_of_month = Some(d);
                }
                _ => {
                    warnings.push("invalid BCD timestamp in step counter header".to_string());
                }
            }
        }

        // Current day steps at offset 374
        let current_day_steps = if payload.len() >= 378 {
            let val = u32::from_le_bytes([payload[374], payload[375], payload[376], payload[377]]);
            if val == SENTINEL_DAILY_VALUE {
                None
            } else {
                Some(val)
            }
        } else {
            None
        };

        // Daily summaries (7 slots x 8 bytes) starting at offset 318
        let mut daily_history = Vec::new();
        let mut daily_distances = Vec::new();
        for i in 0..7 {
            let offset = 318 + i * 8;
            if offset + 8 <= payload.len() {
                let steps = u32::from_le_bytes([
                    payload[offset],
                    payload[offset + 1],
                    payload[offset + 2],
                    payload[offset + 3],
                ]);
                let dist = u32::from_le_bytes([
                    payload[offset + 4],
                    payload[offset + 5],
                    payload[offset + 6],
                    payload[offset + 7],
                ]);

                if steps == SENTINEL_DAILY_VALUE && dist == SENTINEL_DAILY_VALUE {
                    daily_history.push(None);
                    daily_distances.push(None);
                } else {
                    daily_history.push(if steps == SENTINEL_DAILY_VALUE {
                        None
                    } else {
                        Some(steps)
                    });
                    daily_distances.push(if dist == SENTINEL_DAILY_VALUE {
                        None
                    } else {
                        Some(dist)
                    });
                }
            }
        }

        // Activity steps
        let mut hourly_steps = Vec::new();
        for offset in (6..246).step_by(10) {
            if offset + 10 <= payload.len() {
                let mut sum = 0u32;
                let mut valid = false;
                for b_idx in 0..5 {
                    let bucket = u16::from_le_bytes([
                        payload[offset + b_idx * 2],
                        payload[offset + b_idx * 2 + 1],
                    ]);
                    if bucket != SENTINEL_BUCKET_VALUE {
                        sum += u32::from(bucket);
                        valid = true;
                    }
                }
                hourly_steps.push(if valid && sum > 0 { Some(sum) } else { None });
            }
        }

        Some(Self {
            year,
            month,
            day_of_month,
            day_of_week: None,
            hourly_steps,
            daily_history,
            daily_distances,
            current_day_steps,
            distance_meters: Some(0),
            pending_distance_meters: Some(0),
            raw: Some(payload.to_vec()),
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_counter_data_and_parse() {
        let unavail = StepCounterData::unavailable();
        assert_eq!(unavail.current_day_steps, None);
        assert_eq!(unavail.hourly_steps, vec![]);

        let mut payload = vec![0u8; 400];
        payload[0] = 0x26; // Header
        payload[1] = 0x01; // Month
        payload[2] = 0x08; // Day 8

        payload[374..378].copy_from_slice(&12345u32.to_le_bytes());

        let parsed = StepCounterData::parse(&payload).unwrap();
        assert_eq!(parsed.month, Some(1));
        assert_eq!(parsed.day_of_month, Some(8));
        assert_eq!(parsed.current_day_steps, Some(12345));
    }
}
