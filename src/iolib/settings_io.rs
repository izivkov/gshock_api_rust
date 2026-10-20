use crate::iolib::actions::{BleAction, Write};
use crate::iolib::packet::Protocol;

pub use crate::model::settings::{BasicSettings, DateFormat, Language, LightDuration, Settings};

/// Encodes the standard 12-byte basic-settings payload.
pub fn encode_settings(settings: Settings, long_duration: LightDuration) -> [u8; 12] {
    let mut bytes = [0; 12];
    bytes[0] = Protocol::SettingForBasic as u8;
    if settings.twenty_four_hour_time {
        bytes[1] |= 0b0000_0001;
    }
    if !settings.button_tone {
        bytes[1] |= 0b0000_0010;
    }
    if !settings.auto_light {
        bytes[1] |= 0b0000_0100;
    }
    if !settings.power_saving_mode {
        bytes[1] |= 0b0001_0000;
    }
    if settings.light_duration == long_duration {
        bytes[2] = 1;
    }
    if settings.date_format == DateFormat::DdMm {
        bytes[4] = 1;
    }
    bytes[5] = language_byte(settings.language);
    bytes
}

/// Encodes the MTG-B3000 subset. Unsupported wire fields retain observed defaults.
pub fn encode_mtg_b3000_settings(
    settings: BasicSettings,
    long_duration: LightDuration,
) -> [u8; 12] {
    let mut bytes = [0; 12];
    bytes[0] = Protocol::SettingForBasic as u8;
    bytes[1] |= 0b0000_0001;
    if !settings.button_tone {
        bytes[1] |= 0b0000_0010;
    }
    if !settings.power_saving_mode {
        bytes[1] |= 0b0001_0000;
    }
    if settings.light_duration == long_duration {
        bytes[2] = 1;
    }
    bytes
}

fn language_byte(language: Language) -> u8 {
    match language {
        Language::English => 0,
        Language::Spanish => 1,
        Language::French => 2,
        Language::German => 3,
        Language::Italian => 4,
        Language::Russian => 5,
    }
}

fn language_from_byte(byte: u8) -> Language {
    match byte {
        1 => Language::Spanish,
        2 => Language::French,
        3 => Language::German,
        4 => Language::Italian,
        5 => Language::Russian,
        _ => Language::English,
    }
}

/// Decodes the standard payload. Returns `None` for a truncated packet.
pub fn decode_settings(
    bytes: &[u8],
    short_duration: LightDuration,
    long_duration: LightDuration,
) -> Option<Settings> {
    if bytes.len() < 6 {
        return None;
    }
    let flags = bytes[1];
    Some(Settings {
        twenty_four_hour_time: flags & 0b0000_0001 != 0,
        button_tone: flags & 0b0000_0010 == 0,
        auto_light: flags & 0b0000_0100 == 0,
        power_saving_mode: flags & 0b0001_0000 == 0,
        light_duration: if bytes[2] == 1 {
            long_duration
        } else {
            short_duration
        },
        date_format: if bytes[4] == 1 {
            DateFormat::DdMm
        } else {
            DateFormat::MmDd
        },
        language: language_from_byte(bytes[5]),
    })
}

pub fn settings_request() -> BleAction {
    BleAction::Write(Write::new(0x000c, [Protocol::SettingForBasic as u8]))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference_settings() -> Settings {
        Settings {
            twenty_four_hour_time: true,
            button_tone: true,
            auto_light: false,
            power_saving_mode: true,
            light_duration: LightDuration::FourSeconds,
            date_format: DateFormat::DdMm,
            language: Language::French,
        }
    }

    #[test]
    fn encodes_and_decodes_python_reference_settings() {
        let encoded = encode_settings(reference_settings(), LightDuration::FourSeconds);
        assert_eq!(encoded, [0x13, 0b0000_0101, 1, 0, 1, 2, 0, 0, 0, 0, 0, 0]);
        assert_eq!(
            decode_settings(
                &encoded,
                LightDuration::TwoSeconds,
                LightDuration::FourSeconds
            ),
            Some(reference_settings())
        );
    }

    #[test]
    fn encodes_mtg_b3000_subset() {
        let encoded = encode_mtg_b3000_settings(
            BasicSettings {
                button_tone: true,
                power_saving_mode: true,
                light_duration: LightDuration::ThreeSeconds,
            },
            LightDuration::ThreeSeconds,
        );
        assert_eq!(encoded[0], 0x13);
        assert_eq!(encoded[1], 1);
        assert_eq!(encoded[2], 1);
    }

    #[test]
    fn rejects_truncated_settings() {
        assert_eq!(
            decode_settings(
                &[0x13; 5],
                LightDuration::TwoSeconds,
                LightDuration::FourSeconds
            ),
            None
        );
    }
}
