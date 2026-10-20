use gshock_api::{
    AppNotification, NotificationType, decode_notification_packet, encode_notification_packet,
    xor_decode_buffer,
};

fn main() {
    println!("=== App Notification Codec Test ===");

    let sample_hex = "05a9ebffffffef0ff7df0ff7df0ff7df0ff7df0ff74992";
    if let Ok(bytes) = hex::decode(sample_hex) {
        let decoded = xor_decode_buffer(&bytes, 255);
        println!("Decoded XOR hex: {}", hex::encode(&decoded));
    }

    let notif = AppNotification::new(
        NotificationType::Calendar,
        "20260516T233000",
        "Calendar",
        "Meeting with Team",
        "Discuss project deliverables",
        "Meeting",
    );

    println!("Original notification: {:?}", notif);

    let packet = encode_notification_packet(&notif);
    println!("Encoded packet len: {} bytes", packet.len());

    let parsed = decode_notification_packet(&packet).expect("Failed to parse packet");
    println!("Parsed notification: {:?}", parsed);

    assert_eq!(notif, parsed);
    println!("=== App Notification Test Passed ===");
}
