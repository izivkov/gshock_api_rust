use gshock_api::*;

#[test]
fn test_dispatcher_standard_protocol_routing() {
    let model_info = resolve_model_info(WatchModel::Gw);

    // Watch Name packet: 0x23 + "G-SHOCK\0"
    let mut watch_name_packet = vec![0x23];
    watch_name_packet.extend_from_slice(b"G-SHOCK\0");

    let dispatched = MessageDispatcher::dispatch(&watch_name_packet, &model_info).unwrap();
    assert_eq!(dispatched, DispatchedPayload::WatchName("G-SHOCK".to_string()));
}

#[test]
fn test_dispatcher_analogue_protocol_routing() {
    let model_info = resolve_model_info(WatchModel::MtgB1000);

    // Wrapped 0x28 analogue packet containing alarm payload 0x15
    let wrapped_alarm_packet = [0x28, 0x01, 0x00, 0x00, 0x15, 0x40, 0x40, 7, 25];

    let dispatched = MessageDispatcher::dispatch(&wrapped_alarm_packet, &model_info).unwrap();
    assert_eq!(
        dispatched,
        DispatchedPayload::Alarms(vec![Alarm::new(7, 25, true, false)])
    );
}

#[test]
fn test_dispatcher_timer_routing() {
    let model_info = resolve_model_info(WatchModel::Gw);
    let timer_packet = [0x18, 1, 1, 5, 0, 0]; // 1 hour, 1 min, 5 secs = 3665s

    let dispatched = MessageDispatcher::dispatch(&timer_packet, &model_info).unwrap();
    assert_eq!(dispatched, DispatchedPayload::Timer(3665));
}
