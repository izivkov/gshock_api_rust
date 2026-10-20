use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

/// Time fields required by the watch's ten-byte current-time payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateTimeParts {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    /// Python's `datetime.weekday()`: Monday is 0 and Sunday is 6.
    pub weekday: u8,
    pub nanosecond: u32,
}

/// Encodes the watch's ten-byte little-endian current-time payload.
pub fn encode_current_time(time: DateTimeParts) -> [u8; 10] {
    let year = time.year.to_le_bytes();
    let fractional = ((u64::from(time.nanosecond) * 256) / 1_000_000_000) as u8;
    [
        year[0],
        year[1],
        time.month,
        time.day,
        time.hour,
        time.minute,
        time.second,
        time.weekday,
        fractional,
        1,
    ]
}

/// Produces the command used to set the current time.
pub fn time_set_command(time: DateTimeParts) -> BleAction {
    let mut data = Vec::with_capacity(11);
    data.push(Protocol::CurrentTime as u8);
    data.extend(encode_current_time(time));
    BleAction::Write(Write::new(0x000e, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_current_time_like_python_reference() {
        let encoded = encode_current_time(DateTimeParts {
            year: 2026,
            month: 5,
            day: 30,
            hour: 8,
            minute: 45,
            second: 30,
            weekday: 5,
            nanosecond: 123_456_000,
        });
        assert_eq!(encoded, [0xea, 0x07, 5, 30, 8, 45, 30, 5, 31, 1]);
    }

    #[test]
    fn time_set_command_has_protocol_prefix_and_write_handle() {
        let command = time_set_command(DateTimeParts {
            year: 2026,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            weekday: 3,
            nanosecond: 0,
        });
        assert_eq!(
            command,
            BleAction::Write(Write::new(
                0x000e,
                vec![0x09, 0xea, 0x07, 1, 1, 0, 0, 0, 3, 0, 1]
            ))
        );
    }
}
