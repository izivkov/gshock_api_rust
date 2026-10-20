use gshock_api::*;

#[test]
fn test_deterministic_time_encoding() {
    let dt = DateTimeParts {
        year: 2026,
        month: 5,
        day: 30,
        hour: 8,
        minute: 45,
        second: 30,
        weekday: 5,
        nanosecond: 123_456_000,
    };
    let encoded = encode_current_time(dt);
    assert_eq!(encoded.len(), 10);
    assert_eq!(encoded[0], 0xEA);
    assert_eq!(encoded[1], 0x07);
    assert_eq!(encoded[2], 5);
    assert_eq!(encoded[3], 30);
    assert_eq!(encoded[4], 8);
    assert_eq!(encoded[5], 45);
    assert_eq!(encoded[6], 30);
    assert_eq!(encoded[7], 5);
    let expected_nano_byte = ((123_456_000u64 * 256) / 1_000_000_000) as u8;
    assert_eq!(encoded[8], expected_nano_byte);
    assert_eq!(encoded[9], 1);
}

#[test]
fn test_alarms_commands() {
    let commands = alarm_request();
    assert_eq!(commands.len(), 2);
    assert_eq!(commands[0], BleAction::Write(Write::new(0x000c, vec![0x15])));
    assert_eq!(commands[1], BleAction::Write(Write::new(0x000c, vec![0x16])));
}

#[test]
fn test_settings_encode_decode() {
    let settings = Settings {
        twenty_four_hour_time: true,
        button_tone: true,
        auto_light: false,
        power_saving_mode: true,
        light_duration: LightDuration::FourSeconds,
        date_format: DateFormat::DdMm,
        language: Language::French,
    };

    let encoded = encode_settings(settings, LightDuration::FourSeconds);
    assert_eq!(encoded.len(), 12);
    assert_eq!(encoded[0], 0x13); // Protocol::SettingForBasic = 0x13

    let decoded = decode_settings(&encoded, LightDuration::TwoSeconds, LightDuration::FourSeconds).unwrap();
    assert_eq!(decoded, settings);
}

#[test]
fn test_timer_encode_decode() {
    let seconds = 3665;
    let encoded = encode_timer(seconds);
    assert_eq!(encoded[0], 0x18); // Protocol::Timer = 0x18
    assert_eq!(encoded[1], 1); // Hours
    assert_eq!(encoded[2], 1); // Minutes
    assert_eq!(encoded[3], 5); // Seconds

    let decoded = decode_timer(&encoded);
    assert_eq!(decoded, seconds);

    assert_eq!(
        timer_request(),
        BleAction::Write(Write::new(0x000c, vec![0x18]))
    );
}

#[test]
fn test_time_adjustment_encode_decode() {
    let mut orig = vec![0u8; 15];
    orig[12] = 0x80;
    orig[13] = 10;

    let encoded = encode_time_adjustment(&orig, true, 25);
    let (adjusted, minutes) = decode_time_adjustment(&encoded).unwrap();
    assert!(adjusted);
    assert_eq!(minutes, 25);
}

#[test]
fn test_dst_for_world_cities_commands() {
    assert_eq!(
        dst_world_cities_request(),
        BleAction::Write(Write::new(0x000c, vec![0x1e]))
    );
}

#[test]
fn test_dst_watch_state_commands() {
    assert_eq!(
        dst_watch_state_request(),
        BleAction::Write(Write::new(0x000c, vec![0x1d]))
    );
}

#[test]
fn test_app_info_commands() {
    assert_eq!(
        app_info_request(),
        BleAction::Write(Write::new(0x000c, vec![0x22]))
    );

    let mut trigger = vec![0x22];
    trigger.extend_from_slice(&[0xff; 10]);
    trigger.push(0x00);

    let response = prepare_app_info_response(&trigger).unwrap();
    assert_eq!(
        response,
        BleAction::Write(Write::new(
            0x000e,
            vec![0x22, 0x34, 0x88, 0xf4, 0xe5, 0xd5, 0xaf, 0xc8, 0x29, 0xe0, 0x6d, 0x02]
        ))
    );
}

#[test]
fn test_world_cities_commands() {
    assert_eq!(
        world_cities_request(),
        BleAction::Write(Write::new(0x000c, vec![0x1f]))
    );
}

#[test]
fn test_button_pressed_decode() {
    let left_press = [
        0x10, 0x17, 0x62, 0x07, 0x38, 0x85, 0xcd, 0x7f, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let button = decode_button_pressed(&left_press);
    assert_eq!(button, WatchButton::LowerLeft);
}

#[test]
fn test_watch_condition_commands() {
    assert_eq!(
        watch_condition_request(),
        BleAction::Write(Write::new(0x000c, vec![0x28]))
    );
}

#[test]
fn test_watch_name_decode() {
    let mut payload = vec![0x23];
    payload.extend_from_slice(b"G-SHOCK\0");
    assert_eq!(decode_watch_name(&payload), "G-SHOCK");
}

#[test]
fn test_mtg_b3000_alarms_and_timer() {
    assert_eq!(
        alarm_request_mtg_b3000(),
        vec![BleAction::Write(Write::new(0x000c, vec![0x15]))]
    );

    let alarm = Alarm::new(7, 25, true, false);
    let encoded_alarm = model::alarms::encode_first_alarm(&alarm);
    assert_eq!(encoded_alarm, vec![0x15, 0x40, 0x40, 7, 25]);

    let timer_encoded = encode_timer_mtg_b3000(600);
    assert_eq!(timer_encoded.len(), 15);
    assert_eq!(timer_encoded[0], 0x18);
    assert_eq!(timer_encoded[1], 0);
    assert_eq!(timer_encoded[2], 10);
    assert_eq!(timer_encoded[3], 0);
    assert_eq!(&timer_encoded[4..], &[0u8; 11]);
}

#[test]
fn test_watch_info_exact_lookup_and_protocols() {
    let bx = resolve_model_info(resolve_model("CASIO GW-BX5600"));
    assert_eq!(bx.model, WatchModel::GwBx5600);
    assert!(bx.has_new_time_format);
    assert_eq!(bx.protocol, ProtocolKind::Mip);

    let mtg = resolve_model_info(resolve_model("CASIO MTG-B1000"));
    assert_eq!(mtg.model, WatchModel::MtgB1000);
    assert!(mtg.has_second_dial);
    assert_eq!(mtg.protocol, ProtocolKind::Analogue);

    let abl = resolve_model_info(resolve_model("CASIO ABL-100WE"));
    assert_eq!(abl.model, WatchModel::Abl100);
    assert!(abl.has_step_counter);
    assert_eq!(abl.protocol, ProtocolKind::Standard);

    let fb = resolve_model_info(resolve_model("CASIO F-B100W"));
    assert_eq!(fb.model, WatchModel::FB100);
    assert!(fb.has_step_counter);

    let gw = resolve_model_info(resolve_model("CASIO GW-B5600"));
    assert_eq!(gw.model, WatchModel::Gw);
    assert_eq!(gw.world_cities_count, 6);
    assert_eq!(gw.protocol, ProtocolKind::Standard);
}

#[test]
fn test_casio_time_zone_helper() {
    let (lat, lon, exact) = get_world_city_coordinates("Europe/Madrid");
    assert!(exact);
    assert!((lat - 41.4548).abs() < 0.0001);
    assert!((lon - 2.2502).abs() < 0.0001);

    let tz = find_time_zone("Europe/London");
    assert_eq!(tz.name, "LONDON");
    assert_eq!(tz.zone_name, "Europe/London");
}
