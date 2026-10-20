#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EventDate {
    pub year: u16,
    pub month: String,
    pub day: u8,
}

impl EventDate {
    pub fn new(year: u16, month: impl Into<String>, day: u8) -> Self {
        Self {
            year,
            month: month.into(),
            day,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatPeriod {
    Never,
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

impl RepeatPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Never => "NEVER",
            Self::Daily => "DAILY",
            Self::Weekly => "WEEKLY",
            Self::Monthly => "MONTHLY",
            Self::Yearly => "YEARLY",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DAILY" => Self::Daily,
            "WEEKLY" => Self::Weekly,
            "MONTHLY" => Self::Monthly,
            "YEARLY" => Self::Yearly,
            _ => Self::Never,
        }
    }
}

pub struct ReminderMasks;

impl ReminderMasks {
    pub const YEARLY_MASK: u8 = 0b0000_1000;
    pub const MONTHLY_MASK: u8 = 0b0001_0000;
    pub const WEEKLY_MASK: u8 = 0b0000_0100;
    pub const ENABLED_MASK: u8 = 0b0000_0001;

    pub const SUNDAY_MASK: u8 = 0b0000_0001;
    pub const MONDAY_MASK: u8 = 0b0000_0010;
    pub const TUESDAY_MASK: u8 = 0b0000_0100;
    pub const WEDNESDAY_MASK: u8 = 0b0000_1000;
    pub const THURSDAY_MASK: u8 = 0b0001_0000;
    pub const FRIDAY_MASK: u8 = 0b0010_0000;
    pub const SATURDAY_MASK: u8 = 0b0100_0000;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub title: String,
    pub start_date: Option<EventDate>,
    pub end_date: Option<EventDate>,
    pub repeat_period: RepeatPeriod,
    pub days_of_week: Vec<String>,
    pub enabled: bool,
    pub incompatible: bool,
    pub selected: bool,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            title: String::new(),
            start_date: None,
            end_date: None,
            repeat_period: RepeatPeriod::Never,
            days_of_week: Vec::new(),
            enabled: false,
            incompatible: false,
            selected: false,
        }
    }
}

/// Encodes title string to an 18-byte array padded with zeroes.
pub fn encode_reminder_title(title: &str) -> [u8; 18] {
    let mut bytes = [0u8; 18];
    let title_bytes = title.as_bytes();
    let len = title_bytes.len().min(18);
    bytes[..len].copy_from_slice(&title_bytes[..len]);
    bytes
}

/// Decodes title string from byte array.
pub fn decode_reminder_title(data: &[u8]) -> String {
    let text = String::from_utf8_lossy(data);
    text.trim_matches('\0').trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode_reminder_title() {
        let title = "Gym Time";
        let encoded = encode_reminder_title(title);
        assert_eq!(encoded.len(), 18);
        assert_eq!(&encoded[..8], title.as_bytes());

        let decoded = decode_reminder_title(&encoded);
        assert_eq!(decoded, "Gym Time");
    }
}
