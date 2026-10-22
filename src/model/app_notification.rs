#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum NotificationType {
    Generic = 0,
    PhoneCallUrgent = 1,
    PhoneCall = 2,
    Email = 3,
    Message = 4,
    Calendar = 5,
    EmailSms = 6,
}

impl TryFrom<u8> for NotificationType {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        match value {
            0 => Ok(Self::Generic),
            1 => Ok(Self::PhoneCallUrgent),
            2 => Ok(Self::PhoneCall),
            3 => Ok(Self::Email),
            4 => Ok(Self::Message),
            5 => Ok(Self::Calendar),
            6 => Ok(Self::EmailSms),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppNotification {
    pub notification_type: NotificationType,
    pub timestamp: String,
    pub app: String,
    pub title: String,
    pub text: String,
    pub short_text: String,
}

impl AppNotification {
    pub fn new(
        notification_type: NotificationType,
        timestamp: impl Into<String>,
        app: impl Into<String>,
        title: impl Into<String>,
        text: impl Into<String>,
        short_text: impl Into<String>,
    ) -> Self {
        let mut notif = Self {
            notification_type,
            timestamp: timestamp.into(),
            app: app.into(),
            title: title.into(),
            text: text.into(),
            short_text: short_text.into(),
        };
        notif.enforce_length_constraints();
        notif
    }

    fn enforce_length_constraints(&mut self) {
        let max_text = 193;
        let max_short_text = 40;
        let max_combined = 206;

        if self.text.as_bytes().len() > max_text {
            self.text = truncate_utf8(&self.text, max_text);
        }
        if self.short_text.as_bytes().len() > max_short_text {
            self.short_text = truncate_utf8(&self.short_text, max_short_text);
        }

        let combined = self.text.as_bytes().len() + self.short_text.as_bytes().len();
        if combined > max_combined {
            let allowed_text = max_combined.saturating_sub(self.short_text.as_bytes().len());
            self.text = truncate_utf8(&self.text, allowed_text);
        }
    }
}

fn truncate_utf8(s: &str, max_bytes: usize) -> String {
    if s.as_bytes().len() <= max_bytes {
        return s.to_string();
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s[..end].to_string()
}

pub fn xor_encode_buffer(bytes: &[u8], key: u8) -> Vec<u8> {
    bytes.iter().map(|b| b ^ key).collect()
}

pub fn xor_decode_buffer(bytes: &[u8], key: u8) -> Vec<u8> {
    xor_encode_buffer(bytes, key)
}

pub fn write_length_prefixed_string(text: &str) -> Vec<u8> {
    let bytes = text.as_bytes();
    let len = bytes.len().min(255) as u8;
    let mut res = Vec::with_capacity(2 + len as usize);
    res.push(len);
    res.push(0x00);
    res.extend_from_slice(&bytes[..len as usize]);
    res
}

pub fn read_length_prefixed_string(buf: &[u8], offset: usize) -> Result<(String, usize), ()> {
    let prefix_end = offset.checked_add(2).ok_or(())?;
    if prefix_end > buf.len() {
        return Err(());
    }
    let len = buf[offset] as usize;
    if buf[offset + 1] != 0x00 {
        return Err(());
    }
    let start = prefix_end;
    let end = start.checked_add(len).ok_or(())?;
    if end > buf.len() {
        return Err(());
    }
    let s = String::from_utf8(buf[start..end].to_vec()).map_err(|_| ())?;
    Ok((s, end))
}

pub fn encode_notification_packet(notif: &AppNotification) -> Vec<u8> {
    let header = [0x00, 0x00, 0x00, 0x00, 0x00, 0x01];
    let mut res = Vec::new();
    res.extend_from_slice(&header);
    res.push(notif.notification_type as u8);
    res.extend_from_slice(notif.timestamp.as_bytes());
    res.extend_from_slice(&write_length_prefixed_string(&notif.app));
    res.extend_from_slice(&write_length_prefixed_string(&notif.title));
    res.extend_from_slice(&write_length_prefixed_string(&notif.short_text));
    res.extend_from_slice(&write_length_prefixed_string(&notif.text));
    res
}

pub fn decode_notification_packet(buf: &[u8]) -> Result<AppNotification, ()> {
    if buf.len() < 6 + 1 + 15 {
        return Err(());
    }
    let mut offset = 6;
    let notif_type = NotificationType::try_from(buf[offset])?;
    offset += 1;

    let timestamp_bytes = &buf[offset..offset + 15];
    if !timestamp_bytes.is_ascii() {
        return Err(());
    }
    let timestamp = String::from_utf8(timestamp_bytes.to_vec()).map_err(|_| ())?;
    offset += 15;

    let (app, next_offset) = read_length_prefixed_string(buf, offset)?;
    offset = next_offset;

    let (title, next_offset) = read_length_prefixed_string(buf, offset)?;
    offset = next_offset;

    let (short_text, next_offset) = read_length_prefixed_string(buf, offset)?;
    offset = next_offset;

    let (text, _) = read_length_prefixed_string(buf, offset)?;

    Ok(AppNotification::new(
        notif_type, timestamp, app, title, text, short_text,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_and_decode_notification_packet() {
        let notif = AppNotification::new(
            NotificationType::Calendar,
            "20260516T233000",
            "Calendar",
            "Meeting",
            "Discuss project",
            "",
        );
        let encoded = encode_notification_packet(&notif);
        let decoded = decode_notification_packet(&encoded).unwrap();
        assert_eq!(decoded, notif);
    }

    #[test]
    fn rejects_invalid_utf8_fields_and_non_ascii_timestamps() {
        assert!(read_length_prefixed_string(&[1, 0, 0xff], 0).is_err());

        let mut packet = encode_notification_packet(&AppNotification::new(
            NotificationType::Generic,
            "20260516T233000",
            "App",
            "Title",
            "Text",
            "",
        ));
        packet[7] = 0xff;
        assert!(decode_notification_packet(&packet).is_err());
    }
}
