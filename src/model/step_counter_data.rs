use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};

pub const PACKET_HEADER_MARKER: u8 = 0x26;
pub const SENTINEL_BUCKET_VALUE: u16 = 0xFFFE;
pub const SENTINEL_DAILY_VALUE: u32 = 0xFFFF_FFFE;

const HEADER_SIZE: usize = 6;
const ACTIVITY_RECORD_SIZE: usize = 10;
const ACTIVITY_SCAN_LIMIT: usize = 146;
const DAILY_SUMMARY_OFFSET: usize = 318;
const DAILY_SUMMARY_COUNT: usize = 7;
const DAILY_SUMMARY_SIZE: usize = 8;
const COMMITTED_DISTANCE_OFFSET: usize = 246;
const COMMITTED_DISTANCE_END: usize = 318;
const CURRENT_STEPS_OFFSET: usize = 374;
const CURRENT_DISTANCE_OFFSET: usize = 378;
const PENDING_INTENSITY_OFFSET: usize = 382;
const PENDING_DISTANCE_OFFSET: usize = 392;
const BCD_TOTAL_OFFSET: usize = 396;

pub fn decode_bcd(byte: u8) -> Result<u8, &'static str> {
    let high = byte / 16;
    let low = byte % 16;
    if high > 9 || low > 9 {
        return Err("invalid BCD byte");
    }
    Ok(high * 10 + low)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HourlyActivity {
    pub index: usize,
    pub start_minute: u8,
    pub end_minute: u8,
    pub steps: Option<u32>,
    pub intensity: [u16; 5],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyHistoryEntry {
    pub days_ago: usize,
    pub steps: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepCounterData {
    pub timestamp: Option<NaiveDateTime>,
    pub year: Option<u16>,
    pub month: Option<u8>,
    pub day_of_month: Option<u8>,
    pub day_of_week: Option<u8>,
    pub hourly_steps: Vec<Option<u32>>,
    pub hourly_intervals: Vec<HourlyActivity>,
    pub hourly_by_hour: Vec<Option<u32>>,
    pub hourly_intensities: Vec<[u16; 5]>,
    pub daily_history: Vec<Option<u32>>,
    pub daily_history_list: Vec<DailyHistoryEntry>,
    pub daily_distances: Vec<Option<u32>>,
    pub current_day_steps: Option<u32>,
    pub distance_meters: Option<u32>,
    pub pending_distance_meters: Option<u32>,
    pub total_distance_meters: Option<u32>,
    pub bcd_total_steps: Option<u32>,
    pub pending_intensity: Vec<u16>,
    pub committed_distances: Vec<u16>,
    pub raw: Option<Vec<u8>>,
    pub warnings: Vec<String>,
}

impl StepCounterData {
    pub fn unavailable() -> Self {
        Self {
            timestamp: None,
            year: None,
            month: None,
            day_of_month: None,
            day_of_week: None,
            hourly_steps: Vec::new(),
            hourly_intervals: Vec::new(),
            hourly_by_hour: Vec::new(),
            hourly_intensities: Vec::new(),
            daily_history: Vec::new(),
            daily_history_list: Vec::new(),
            daily_distances: Vec::new(),
            current_day_steps: None,
            distance_meters: None,
            pending_distance_meters: None,
            total_distance_meters: None,
            bcd_total_steps: None,
            pending_intensity: Vec::new(),
            committed_distances: Vec::new(),
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
        let mut day_of_week = None;
        let mut timestamp = None;

        if payload.len() >= HEADER_SIZE {
            let decoded_year = decode_bcd(payload[0]);
            let decoded_month = decode_bcd(payload[1]);
            let decoded_day = decode_bcd(payload[2]);
            let decoded_hour = decode_bcd(payload[3]);
            let decoded_minute = decode_bcd(payload[4]);
            let decoded_second = decode_bcd(payload[5]);

            if let Ok(value) = decoded_year {
                year = Some(2000 + u16::from(value));
            }
            if let Ok(value) = decoded_month {
                month = Some(value);
            }
            if let Ok(value) = decoded_day {
                day_of_month = Some(value);
            }

            if let (Ok(y), Ok(m), Ok(d), Ok(hour), Ok(minute), Ok(second)) = (
                decoded_year,
                decoded_month,
                decoded_day,
                decoded_hour,
                decoded_minute,
                decoded_second,
            ) {
                timestamp =
                    NaiveDate::from_ymd_opt(2000 + i32::from(y), u32::from(m), u32::from(d))
                        .and_then(|date| {
                            date.and_hms_opt(u32::from(hour), u32::from(minute), u32::from(second))
                        });
            }

            if timestamp.is_none() {
                warnings.push("invalid BCD timestamp in step counter header".to_string());
            } else if let Some(timestamp) = timestamp {
                day_of_week = Some(timestamp.weekday().num_days_from_monday() as u8);
            }
        }

        let current_day_steps =
            read_u32(payload, CURRENT_STEPS_OFFSET).filter(|value| *value != SENTINEL_DAILY_VALUE);

        let pending_intensity = if payload.len() >= PENDING_INTENSITY_OFFSET + 6 {
            read_u16_array::<3>(payload, PENDING_INTENSITY_OFFSET)
        } else {
            Vec::new()
        };
        let pending_steps = pending_intensity
            .iter()
            .filter(|value| **value != SENTINEL_BUCKET_VALUE)
            .map(|value| u32::from(*value))
            .sum::<u32>();

        let record_end = find_activity_record_end(payload, current_day_steps, pending_steps);
        let mut hourly_steps = Vec::new();
        let mut hourly_intensities = Vec::new();
        let mut hourly_intervals = Vec::new();
        for (index, offset) in (HEADER_SIZE..record_end)
            .step_by(ACTIVITY_RECORD_SIZE)
            .enumerate()
        {
            let Some(buckets) = read_bucket_array(payload, offset) else {
                break;
            };
            let sum = buckets
                .iter()
                .filter(|value| **value != SENTINEL_BUCKET_VALUE)
                .map(|value| u32::from(*value))
                .sum::<u32>();
            let steps = (sum > 0).then_some(sum);
            hourly_steps.push(steps);
            hourly_intensities.push(buckets);
            hourly_intervals.push(HourlyActivity {
                index,
                start_minute: 0,
                end_minute: 59,
                steps,
                intensity: buckets,
            });
        }

        let mut daily_history = Vec::new();
        let mut daily_distances = Vec::new();
        for index in 0..DAILY_SUMMARY_COUNT {
            let offset = DAILY_SUMMARY_OFFSET + index * DAILY_SUMMARY_SIZE;
            let (Some(steps), Some(distance)) =
                (read_u32(payload, offset), read_u32(payload, offset + 4))
            else {
                break;
            };
            daily_history.push((steps != SENTINEL_DAILY_VALUE).then_some(steps));
            daily_distances.push((distance != SENTINEL_DAILY_VALUE).then_some(distance));
        }

        let daily_history_list = daily_history
            .iter()
            .enumerate()
            .map(|(index, steps)| DailyHistoryEntry {
                days_ago: index + 1,
                steps: *steps,
            })
            .collect();

        let distance_meters = read_u32(payload, CURRENT_DISTANCE_OFFSET);
        let pending_distance_meters = read_u32(payload, PENDING_DISTANCE_OFFSET);
        let total_distance_meters = distance_meters;
        let committed_distances = decode_committed_distances(
            payload,
            distance_meters,
            pending_distance_meters,
            &mut warnings,
        );

        let bcd_total_steps = decode_bcd_total(payload);
        if let (Some(total), Some(current)) = (bcd_total_steps, current_day_steps) {
            if total > 0 && total != current {
                warnings.push(format!(
                    "BCD total {total} differs from current step count {current}"
                ));
            }
        }

        if current_day_steps.is_none() && payload.len() < CURRENT_STEPS_OFFSET + 4 {
            warnings.push("step record truncated; missing trailing history fields".to_string());
        }

        let mut hourly_by_hour = vec![None; 24];
        if let Some(timestamp) = timestamp {
            for (index, steps) in hourly_steps.iter().enumerate() {
                if let Some(steps) = steps {
                    let hour = (timestamp.hour() as usize + 24 - (index + 1) % 24) % 24;
                    hourly_by_hour[hour] = Some(*steps);
                }
            }
            if pending_steps > 0 {
                hourly_by_hour[timestamp.hour() as usize] = Some(pending_steps);
            }
        }

        Some(Self {
            timestamp,
            year,
            month,
            day_of_month,
            day_of_week,
            hourly_steps,
            hourly_intervals,
            hourly_by_hour,
            hourly_intensities,
            daily_history,
            daily_history_list,
            daily_distances,
            current_day_steps,
            distance_meters,
            pending_distance_meters,
            total_distance_meters,
            bcd_total_steps,
            pending_intensity,
            committed_distances,
            raw: Some(payload.to_vec()),
            warnings,
        })
    }
}

fn read_u16(payload: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        payload.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        payload.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u16_array<const N: usize>(payload: &[u8], offset: usize) -> Vec<u16> {
    (0..N)
        .filter_map(|index| read_u16(payload, offset + index * 2))
        .collect()
}

fn read_bucket_array(payload: &[u8], offset: usize) -> Option<[u16; 5]> {
    let mut buckets = [0; 5];
    for (index, bucket) in buckets.iter_mut().enumerate() {
        *bucket = read_u16(payload, offset + index * 2)?;
    }
    Some(buckets)
}

fn find_activity_record_end(
    payload: &[u8],
    current_steps: Option<u32>,
    pending_steps: u32,
) -> usize {
    let available_end = payload.len().min(ACTIVITY_SCAN_LIMIT - 1);
    let complete_records = available_end.saturating_sub(HEADER_SIZE) / ACTIVITY_RECORD_SIZE;
    let Some(current_steps) = current_steps else {
        return HEADER_SIZE;
    };

    let mut best = (u32::MAX, HEADER_SIZE);
    for record_count in 0..=complete_records {
        let end = HEADER_SIZE + record_count * ACTIVITY_RECORD_SIZE;
        let mut front_total = 0u32;
        for offset in (HEADER_SIZE..end).step_by(ACTIVITY_RECORD_SIZE) {
            if let Some(buckets) = read_bucket_array(payload, offset) {
                front_total += buckets
                    .iter()
                    .filter(|value| **value != SENTINEL_BUCKET_VALUE)
                    .map(|value| u32::from(*value))
                    .sum::<u32>();
            }
        }
        let observed = pending_steps.saturating_add(front_total);
        let difference = current_steps.abs_diff(observed);
        if difference < best.0 {
            best = (difference, end);
        }
    }
    best.1
}

fn decode_committed_distances(
    payload: &[u8],
    total_distance: Option<u32>,
    pending_distance: Option<u32>,
    warnings: &mut Vec<String>,
) -> Vec<u16> {
    let (Some(total), Some(pending)) = (total_distance, pending_distance) else {
        return Vec::new();
    };
    if total < pending {
        return Vec::new();
    }
    let target = total - pending;
    if target == 0 {
        return Vec::new();
    }

    let mut distances = Vec::new();
    let mut sum = 0u32;
    for offset in (COMMITTED_DISTANCE_OFFSET..COMMITTED_DISTANCE_END).step_by(2) {
        let Some(value) = read_u16(payload, offset) else {
            break;
        };
        if value == SENTINEL_BUCKET_VALUE {
            continue;
        }
        distances.push(value);
        sum += u32::from(value);
        if sum == target {
            return distances;
        }
        if sum > target {
            break;
        }
    }

    warnings.push(format!(
        "distance components do not reconcile to {target} m"
    ));
    Vec::new()
}

fn decode_bcd_total(payload: &[u8]) -> Option<u32> {
    let bytes = payload.get(BCD_TOTAL_OFFSET..BCD_TOTAL_OFFSET + 4)?;
    if bytes.iter().all(|byte| *byte == 0) {
        return None;
    }
    let mut total = 0u32;
    for (index, byte) in bytes.iter().enumerate() {
        total = total.checked_add(u32::from(decode_bcd(*byte).ok()?) * 100u32.pow(index as u32))?;
    }
    Some(total)
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

    #[test]
    fn parses_python_lifelog_fixture_fields() {
        let mut payload = vec![0u8; 400];
        payload[..6].copy_from_slice(&[0x26, 0x01, 0x08, 0x17, 0x30, 0x00]);

        for index in 0..144 {
            let offset = 6 + index * 2;
            payload[offset..offset + 2].copy_from_slice(&(10u16 + index as u16).to_le_bytes());
        }
        for index in 0..14 {
            let offset = 318 + index * 4;
            payload[offset..offset + 4].copy_from_slice(&(5000u32 + index as u32).to_le_bytes());
        }
        payload[374..378].copy_from_slice(&12345u32.to_le_bytes());

        let parsed = StepCounterData::parse(&payload).unwrap();
        assert_eq!(
            parsed.timestamp,
            NaiveDate::from_ymd_opt(2026, 1, 8)
                .unwrap()
                .and_hms_opt(17, 30, 0)
        );
        assert_eq!(parsed.day_of_week, Some(3));
        assert_eq!(parsed.hourly_steps.len(), 13);
        assert_eq!(parsed.hourly_steps[0], Some(60));
        assert_eq!(parsed.hourly_intensities[0], [10, 11, 12, 13, 14]);
        assert_eq!(parsed.daily_history.len(), 7);
        assert_eq!(parsed.daily_history[0], Some(5000));
        assert_eq!(parsed.daily_distances[0], Some(5001));
        assert_eq!(parsed.current_day_steps, Some(12345));
        assert_eq!(parsed.distance_meters, Some(0));
        assert_eq!(parsed.pending_distance_meters, Some(0));
        assert_eq!(parsed.pending_intensity, vec![0, 0, 0]);
        assert!(parsed.warnings.is_empty());
    }

    #[test]
    fn malformed_and_truncated_payloads_are_reported_without_panicking() {
        let malformed = StepCounterData::parse(&[0x26, 0xff, 0xff, 0xff, 0xff, 0xff]).unwrap();
        assert!(malformed.timestamp.is_none());
        assert!(
            malformed
                .warnings
                .iter()
                .any(|warning| warning.contains("invalid BCD timestamp"))
        );

        let truncated = StepCounterData::parse(&[0x26, 0x01, 0x08]).unwrap();
        assert!(
            truncated
                .warnings
                .iter()
                .any(|warning| warning.contains("truncated"))
        );
        assert!(truncated.pending_intensity.is_empty());
    }

    #[test]
    fn decodes_committed_distance_stack_and_bcd_total() {
        let mut payload = vec![0u8; 400];
        payload[..6].copy_from_slice(&[0x26, 0x01, 0x08, 0x17, 0x30, 0x00]);
        payload[246..248].copy_from_slice(&30u16.to_le_bytes());
        payload[248..250].copy_from_slice(&50u16.to_le_bytes());
        payload[374..378].copy_from_slice(&1234u32.to_le_bytes());
        payload[378..382].copy_from_slice(&100u32.to_le_bytes());
        payload[392..396].copy_from_slice(&20u32.to_le_bytes());
        payload[396..400].copy_from_slice(&[0x34, 0x12, 0x00, 0x00]);

        let parsed = StepCounterData::parse(&payload).unwrap();
        assert_eq!(parsed.committed_distances, vec![30, 50]);
        assert_eq!(parsed.distance_meters, Some(100));
        assert_eq!(parsed.pending_distance_meters, Some(20));
        assert_eq!(parsed.total_distance_meters, Some(100));
        assert_eq!(parsed.bcd_total_steps, Some(1234));
        assert!(parsed.warnings.is_empty());
    }
}
