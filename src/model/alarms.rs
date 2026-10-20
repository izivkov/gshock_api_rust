use crate::Protocol;

pub const HOURLY_CHIME_MASK: u8 = 0b1000_0000;
pub const ENABLED_MASK: u8 = 0b0100_0000;
pub const ALARM_CONSTANT_VALUE: u8 = 0x40;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Alarm {
    pub hour: u8,
    pub minute: u8,
    pub enabled: bool,
    pub has_hourly_chime: bool,
}

impl Alarm {
    pub fn new(hour: u8, minute: u8, enabled: bool, has_hourly_chime: bool) -> Self {
        Self {
            hour,
            minute,
            enabled,
            has_hourly_chime,
        }
    }
}

pub fn encode_first_alarm(alarm: &Alarm) -> Vec<u8> {
    let mut flag = 0u8;
    if alarm.enabled {
        flag |= ENABLED_MASK;
    }
    if alarm.has_hourly_chime {
        flag |= HOURLY_CHIME_MASK;
    }
    vec![
        Protocol::SettingForAlm as u8,
        flag,
        ALARM_CONSTANT_VALUE,
        alarm.hour,
        alarm.minute,
    ]
}

pub fn encode_secondary_alarms(alarms: &[Alarm]) -> Vec<u8> {
    if alarms.len() < 2 {
        return vec![];
    }
    let mut result = vec![Protocol::SettingForAlm2 as u8];
    for alarm in &alarms[1..] {
        let mut flag = 0u8;
        if alarm.enabled {
            flag |= ENABLED_MASK;
        }
        if alarm.has_hourly_chime {
            flag |= HOURLY_CHIME_MASK;
        }
        result.extend_from_slice(&[flag, ALARM_CONSTANT_VALUE, alarm.hour, alarm.minute]);
    }
    result
}

pub fn decode_alarm(data: &[u8]) -> Option<Alarm> {
    if data.len() < 4 {
        return None;
    }
    let flag = data[0];
    Some(Alarm {
        enabled: (flag & ENABLED_MASK) != 0,
        has_hourly_chime: (flag & HOURLY_CHIME_MASK) != 0,
        hour: data[2],
        minute: data[3],
    })
}

pub fn decode_alarms_payload(data: &[u8]) -> Vec<Alarm> {
    if data.is_empty() {
        return vec![];
    }
    let mut alarms = Vec::new();
    let cmd = data[0];
    let payload = &data[1..];

    if cmd == Protocol::SettingForAlm as u8 {
        if let Some(alarm) = decode_alarm(payload) {
            alarms.push(alarm);
        }
    } else if cmd == Protocol::SettingForAlm2 as u8 {
        let quarter_len = payload.len() / 4;
        for i in 0..quarter_len {
            let start = i * 4;
            if start + 4 <= payload.len() {
                if let Some(alarm) = decode_alarm(&payload[start..start + 4]) {
                    alarms.push(alarm);
                }
            }
        }
    }
    alarms
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode_alarm() {
        let alarm = Alarm::new(7, 25, true, false);
        let encoded = encode_first_alarm(&alarm);
        assert_eq!(encoded, vec![0x15, 0x40, 0x40, 7, 25]);

        let decoded = decode_alarm(&encoded[1..]).unwrap();
        assert_eq!(decoded, alarm);
    }
}
