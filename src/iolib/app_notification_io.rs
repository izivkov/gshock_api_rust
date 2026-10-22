pub use crate::model::app_notification::{
    AppNotification, NotificationType, decode_notification_packet, encode_notification_packet,
    read_length_prefixed_string, write_length_prefixed_string, xor_decode_buffer,
    xor_encode_buffer,
};

use crate::casio_constants::HANDLE_ALL_FEATURES_NOTIFICATION;
use crate::iolib::actions::{BleAction, Write};
use crate::model::app_notification::xor_encode_buffer as xor_encode;

pub const NOTIFICATION_XOR_KEY: u8 = 0xff;

pub fn notification_write(notification: &AppNotification) -> Result<BleAction, &'static str> {
    if notification.timestamp.len() != 15 || !notification.timestamp.is_ascii() {
        return Err("notification timestamp must be 15 ASCII bytes");
    }

    for value in [
        &notification.app,
        &notification.title,
        &notification.short_text,
        &notification.text,
    ] {
        if value.len() > u8::MAX as usize {
            return Err("notification string exceeds the 255-byte protocol limit");
        }
    }

    let packet = encode_notification_packet(notification);
    let encrypted = xor_encode(&packet, NOTIFICATION_XOR_KEY);
    Ok(BleAction::Write(Write::new(
        HANDLE_ALL_FEATURES_NOTIFICATION,
        encrypted,
    )))
}

pub fn decode_watch_notification(data: &[u8]) -> Result<AppNotification, ()> {
    let decoded = xor_decode_buffer(data, NOTIFICATION_XOR_KEY);
    decode_notification_packet(&decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepares_encrypted_notification_write_on_notification_handle() {
        let notification = AppNotification::new(
            NotificationType::Calendar,
            "20260516T233000",
            "Calendar",
            "Meeting",
            "Discuss project",
            "",
        );

        let BleAction::Write(write) = notification_write(&notification).unwrap() else {
            panic!("notification must produce a write action");
        };
        assert_eq!(write.handle, HANDLE_ALL_FEATURES_NOTIFICATION);
        assert_eq!(
            xor_decode_buffer(&write.data, NOTIFICATION_XOR_KEY),
            encode_notification_packet(&notification)
        );
    }

    #[test]
    fn rejects_invalid_timestamp_and_oversized_strings() {
        let mut notification = AppNotification::new(
            NotificationType::Message,
            "bad",
            "Messages",
            "Title",
            "Text",
            "",
        );
        assert!(notification_write(&notification).is_err());

        notification.timestamp = "20260516T233000".to_string();
        notification.app = "x".repeat(256);
        assert!(notification_write(&notification).is_err());
    }

    #[test]
    fn decodes_watch_encrypted_notification() {
        let notification = AppNotification::new(
            NotificationType::Email,
            "20260516T233000",
            "Mail",
            "Subject",
            "Body",
            "Short",
        );
        let encoded = xor_encode_buffer(
            &encode_notification_packet(&notification),
            NOTIFICATION_XOR_KEY,
        );
        assert_eq!(decode_watch_notification(&encoded), Ok(notification));
    }
}
