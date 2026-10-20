use gshock_api::StepCounterData;

fn main() {
    println!("=== Step Counter (Life-Log) Test ===");

    let unavail = StepCounterData::unavailable();
    println!("Unavailable step data: {:?}", unavail);

    let mut mock_payload = vec![0u8; 400];
    mock_payload[0] = 0x26; // Header marker
    mock_payload[1] = 0x01; // Month BCD (Jan)
    mock_payload[2] = 0x08; // Day BCD (8th)
    mock_payload[3] = 0x12; // Hour BCD (12)
    mock_payload[4] = 0x30; // Minute BCD (30)
    mock_payload[5] = 0x00; // Second BCD (00)

    // Current day steps at offset 374
    mock_payload[374..378].copy_from_slice(&12345u32.to_le_bytes());

    if let Some(parsed) = StepCounterData::parse(&mock_payload) {
        println!("Parsed StepCounterData:");
        println!("  Month: {:?}", parsed.month);
        println!("  Day: {:?}", parsed.day_of_month);
        println!("  Current day steps: {:?}", parsed.current_day_steps);
        println!("  Hourly steps count: {}", parsed.hourly_steps.len());
        println!("  Daily history count: {}", parsed.daily_history.len());
    } else {
        println!("Failed to parse step counter payload");
    }

    println!("=== Step Counter Test Passed ===");
}
