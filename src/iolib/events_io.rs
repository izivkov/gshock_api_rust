pub use crate::model::event::{
    decode_reminder_title, encode_reminder_title, Event, EventDate, ReminderMasks, RepeatPeriod,
};

use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

const REMINDER_TITLE_SIZE: usize = 18;
const REMINDER_TIME_DETAIL_SIZE: usize = 8;

pub fn reminder_title_request(event_number: u8) -> BleAction {
    BleAction::Write(Write::new(
        0x000c,
        [Protocol::ReminderTitle as u8, event_number],
    ))
}

pub fn reminder_time_request(event_number: u8) -> BleAction {
    BleAction::Write(Write::new(
        0x000c,
        [Protocol::ReminderTime as u8, event_number],
    ))
}

pub fn encode_reminder_title_packet(event_number: u8, title: &str) -> Vec<u8> {
    let mut packet = Vec::with_capacity(2 + REMINDER_TITLE_SIZE);
    packet.extend_from_slice(&[Protocol::ReminderTitle as u8, event_number]);
    packet.extend_from_slice(&encode_reminder_title(title));
    packet
}

pub fn encode_reminder_time(event: &Event) -> [u8; 1 + REMINDER_TIME_DETAIL_SIZE] {
    let period = u8::from(event.enabled) | repeat_period_mask(event.repeat_period);
    let mut detail = [0; REMINDER_TIME_DETAIL_SIZE];

    if matches!(
        event.repeat_period,
        RepeatPeriod::Never | RepeatPeriod::Weekly | RepeatPeriod::Monthly | RepeatPeriod::Yearly
    ) {
        encode_date(&mut detail, 0, event.start_date.as_ref());
        encode_date(&mut detail, 3, event.end_date.as_ref());
    }

    if event.repeat_period == RepeatPeriod::Weekly {
        detail[6] = event
            .days_of_week
            .iter()
            .fold(0, |mask, day| mask | weekday_mask(day));
    }

    let mut result = [0; 1 + REMINDER_TIME_DETAIL_SIZE];
    result[0] = period;
    result[1..].copy_from_slice(&detail);
    result
}

pub fn encode_reminder_time_packet(event_number: u8, event: &Event) -> Vec<u8> {
    let time = encode_reminder_time(event);
    let mut packet = Vec::with_capacity(2 + time.len());
    packet.extend_from_slice(&[Protocol::ReminderTime as u8, event_number]);
    packet.extend_from_slice(&time);
    packet
}

pub fn reminder_set_commands(events: &[Event]) -> Vec<BleAction> {
    events
        .iter()
        .filter(|event| event.enabled)
        .enumerate()
        .flat_map(|(index, event)| {
            let event_number = (index + 1) as u8;
            [
                BleAction::Write(Write::new(
                    0x000e,
                    encode_reminder_title_packet(event_number, &event.title),
                )),
                BleAction::Write(Write::new(
                    0x000e,
                    encode_reminder_time_packet(event_number, event),
                )),
            ]
        })
        .collect()
}

pub fn decode_reminder_time(
    data: &[u8],
) -> Option<(bool, RepeatPeriod, EventDate, EventDate, Vec<String>)> {
    if data.len() < 1 + REMINDER_TIME_DETAIL_SIZE {
        return None;
    }

    let period = data[0];
    let repeat_period = if period & ReminderMasks::WEEKLY_MASK != 0 {
        RepeatPeriod::Weekly
    } else if period & ReminderMasks::MONTHLY_MASK != 0 {
        RepeatPeriod::Monthly
    } else if period & ReminderMasks::YEARLY_MASK != 0 {
        RepeatPeriod::Yearly
    } else {
        RepeatPeriod::Never
    };
    let detail = &data[1..1 + REMINDER_TIME_DETAIL_SIZE];
    let start_date = decode_date(detail, 0)?;
    let end_date = decode_date(detail, 3)?;
    let days_of_week = if repeat_period == RepeatPeriod::Weekly {
        decode_weekdays(detail[6])
    } else {
        Vec::new()
    };

    Some((
        period & ReminderMasks::ENABLED_MASK != 0,
        repeat_period,
        start_date,
        end_date,
        days_of_week,
    ))
}

pub fn decode_reminder_time_packet(data: &[u8]) -> Option<(u8, Event)> {
    if data.len() < 2 || data[0] != Protocol::ReminderTime as u8 {
        return None;
    }
    let (enabled, repeat_period, start_date, end_date, days_of_week) =
        decode_reminder_time(&data[2..])?;
    Some((
        data[1],
        Event {
            start_date: Some(start_date),
            end_date: Some(end_date),
            repeat_period,
            days_of_week,
            enabled,
            ..Event::default()
        },
    ))
}

pub fn decode_reminder_title_packet(data: &[u8]) -> Option<(u8, Option<String>)> {
    if data.len() < 2 || data[0] != Protocol::ReminderTitle as u8 {
        return None;
    }
    if data.get(2) == Some(&0xff) {
        return Some((data[1], None));
    }
    Some((data[1], Some(decode_reminder_title(&data[2..]))))
}

fn repeat_period_mask(repeat_period: RepeatPeriod) -> u8 {
    match repeat_period {
        RepeatPeriod::Weekly => ReminderMasks::WEEKLY_MASK,
        RepeatPeriod::Monthly => ReminderMasks::MONTHLY_MASK,
        RepeatPeriod::Yearly => ReminderMasks::YEARLY_MASK,
        RepeatPeriod::Never | RepeatPeriod::Daily => 0,
    }
}

fn encode_date(
    detail: &mut [u8; REMINDER_TIME_DETAIL_SIZE],
    offset: usize,
    date: Option<&EventDate>,
) {
    let (year, month, day) = match date {
        Some(date) => (date.year % 2000, month_number(&date.month), date.day),
        None => (0, 1, 0),
    };
    detail[offset] = to_bcd(year as u8);
    detail[offset + 1] = to_bcd(month);
    detail[offset + 2] = to_bcd(day);
}

fn decode_date(detail: &[u8], offset: usize) -> Option<EventDate> {
    Some(EventDate::new(
        2000 + u16::from(from_bcd(*detail.get(offset)?)?),
        month_name(from_bcd(*detail.get(offset + 1)?)?),
        from_bcd(*detail.get(offset + 2)?)?,
    ))
}

fn month_number(month: &str) -> u8 {
    match month.to_ascii_lowercase().as_str() {
        "january" => 1,
        "february" => 2,
        "march" => 3,
        "april" => 4,
        "may" => 5,
        "june" => 6,
        "july" => 7,
        "august" => 8,
        "september" => 9,
        "october" => 10,
        "november" => 11,
        "december" => 12,
        _ => 1,
    }
}

fn month_name(month: u8) -> String {
    match month {
        1 => "JANUARY",
        2 => "FEBRUARY",
        3 => "MARCH",
        4 => "APRIL",
        5 => "MAY",
        6 => "JUNE",
        7 => "JULY",
        8 => "AUGUST",
        9 => "SEPTEMBER",
        10 => "OCTOBER",
        11 => "NOVEMBER",
        12 => "DECEMBER",
        _ => "",
    }
    .to_string()
}

fn weekday_mask(day: &str) -> u8 {
    match day {
        "SUNDAY" => ReminderMasks::SUNDAY_MASK,
        "MONDAY" => ReminderMasks::MONDAY_MASK,
        "TUESDAY" => ReminderMasks::TUESDAY_MASK,
        "WEDNESDAY" => ReminderMasks::WEDNESDAY_MASK,
        "THURSDAY" => ReminderMasks::THURSDAY_MASK,
        "FRIDAY" => ReminderMasks::FRIDAY_MASK,
        "SATURDAY" => ReminderMasks::SATURDAY_MASK,
        _ => 0,
    }
}

fn decode_weekdays(mask: u8) -> Vec<String> {
    [
        (ReminderMasks::SUNDAY_MASK, "SUNDAY"),
        (ReminderMasks::MONDAY_MASK, "MONDAY"),
        (ReminderMasks::TUESDAY_MASK, "TUESDAY"),
        (ReminderMasks::WEDNESDAY_MASK, "WEDNESDAY"),
        (ReminderMasks::THURSDAY_MASK, "THURSDAY"),
        (ReminderMasks::FRIDAY_MASK, "FRIDAY"),
        (ReminderMasks::SATURDAY_MASK, "SATURDAY"),
    ]
    .into_iter()
    .filter_map(|(bit, day)| (mask & bit != 0).then(|| day.to_string()))
    .collect()
}

fn to_bcd(value: u8) -> u8 {
    (value / 10) << 4 | (value % 10)
}

fn from_bcd(value: u8) -> Option<u8> {
    let high = value >> 4;
    let low = value & 0x0f;
    (high <= 9 && low <= 9).then_some(high * 10 + low)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_reminder_packets_like_python_format() {
        let event = Event {
            title: "GYM".to_string(),
            start_date: Some(EventDate::new(2026, "MAY", 30)),
            end_date: Some(EventDate::new(2026, "JUNE", 1)),
            repeat_period: RepeatPeriod::Weekly,
            days_of_week: vec!["MONDAY".to_string(), "FRIDAY".to_string()],
            enabled: true,
            ..Event::default()
        };

        assert_eq!(
            encode_reminder_title_packet(1, "GYM")[..5],
            [0x30, 1, b'G', b'Y', b'M']
        );
        assert_eq!(
            encode_reminder_time(&event),
            [0x05, 0x26, 0x05, 0x30, 0x26, 0x06, 0x01, 0x22, 0x00]
        );
    }

    #[test]
    fn decodes_reminder_time_and_weekdays() {
        let decoded =
            decode_reminder_time(&[0x05, 0x26, 0x05, 0x30, 0x26, 0x06, 0x01, 0x22, 0x00]).unwrap();
        assert_eq!(decoded.0, true);
        assert_eq!(decoded.1, RepeatPeriod::Weekly);
        assert_eq!(decoded.2, EventDate::new(2026, "MAY", 30));
        assert_eq!(decoded.3, EventDate::new(2026, "JUNE", 1));
        assert_eq!(decoded.4, vec!["MONDAY", "FRIDAY"]);
    }

    #[test]
    fn decodes_time_packet_and_generates_set_commands() {
        let event = Event {
            title: "GYM".to_string(),
            start_date: Some(EventDate::new(2026, "MAY", 30)),
            end_date: Some(EventDate::new(2026, "JUNE", 1)),
            repeat_period: RepeatPeriod::Weekly,
            days_of_week: vec!["MONDAY".to_string(), "FRIDAY".to_string()],
            enabled: true,
            ..Event::default()
        };
        let time_packet = encode_reminder_time_packet(2, &event);
        let (event_number, decoded) = decode_reminder_time_packet(&time_packet).unwrap();

        assert_eq!(event_number, 2);
        assert_eq!(decoded.start_date, event.start_date);
        assert_eq!(decoded.repeat_period, event.repeat_period);
        assert_eq!(decoded.days_of_week, event.days_of_week);

        let commands = reminder_set_commands(&[Event::default(), event]);
        assert_eq!(commands.len(), 2);
        assert_eq!(
            commands[0],
            BleAction::Write(Write::new(0x000e, encode_reminder_title_packet(1, "GYM")))
        );
        assert_eq!(
            commands[1],
            BleAction::Write(Write::new(
                0x000e,
                encode_reminder_time_packet(
                    1,
                    &Event {
                        enabled: true,
                        title: "GYM".to_string(),
                        start_date: Some(EventDate::new(2026, "MAY", 30)),
                        end_date: Some(EventDate::new(2026, "JUNE", 1)),
                        repeat_period: RepeatPeriod::Weekly,
                        days_of_week: vec!["MONDAY".to_string(), "FRIDAY".to_string()],
                        ..Event::default()
                    }
                )
            ))
        );
    }
}
