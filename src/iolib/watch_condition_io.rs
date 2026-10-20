use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WatchConditionValue {
    pub battery_level_percent: u8,
    pub temperature: i8,
}

pub fn watch_condition_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::WatchCondition as u8]))
}

pub fn decode_watch_condition(data: &[u8], lower_limit: u8, upper_limit: u8) -> WatchConditionValue {
    if data.len() < 3 || data[0] != Protocol::WatchCondition as u8 {
        return WatchConditionValue {
            battery_level_percent: 0,
            temperature: 0,
        };
    }

    let raw_battery = data[1];
    let temperature = data[2] as i8;

    let range = upper_limit.saturating_sub(lower_limit) as f32;
    let percent = if range > 0.0 && raw_battery >= lower_limit {
        let pct = ((f32::from(raw_battery - lower_limit) / range) * 100.0).round() as u8;
        pct.min(100)
    } else {
        0
    };

    WatchConditionValue {
        battery_level_percent: percent,
        temperature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_condition() {
        assert_eq!(
            watch_condition_request(),
            BleAction::Write(Write::new(0x000c, [0x28]))
        );

        let data = [0x28, 14, 22]; // lower 9, upper 19 -> (14-9)/10 * 100 = 50%
        let cond = decode_watch_condition(&data, 9, 19);
        assert_eq!(cond.battery_level_percent, 50);
        assert_eq!(cond.temperature, 22);
    }
}
