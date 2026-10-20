use crate::iolib::{
    BleAction, WatchButton, WatchConditionValue, alarm_request, app_info_request, button_request,
    decode_button_pressed, decode_settings, decode_time_adjustment, decode_timer,
    decode_watch_condition, decode_watch_name, home_time_request, parse_home_city,
    prepare_app_info_response, settings_request, time_adjustment_request, timer_request,
    watch_condition_request, watch_name_request,
};
use crate::model::{
    Alarm, LightDuration, Settings, StepCounterData, alarms::decode_alarms_payload,
};
use crate::protocol_handler;
use crate::watch_info::ModelInfo;

#[derive(Clone, Debug, PartialEq)]
pub enum DispatchedPayload {
    Alarms(Vec<Alarm>),
    Timer(u32),
    WatchName(String),
    DstWorldCities(Vec<u8>),
    WorldCities(Vec<u8>),
    DstWatchState(Vec<u8>),
    WatchCondition(WatchConditionValue),
    AppInfoResponse(Option<BleAction>),
    ButtonPressed(WatchButton),
    Settings(Option<Settings>),
    TimeAdjustment(Option<(bool, u8)>),
    StepCounter(Option<StepCounterData>),
    HomeTime(String),
    Unknown(u8, Vec<u8>),
}

pub struct MessageDispatcher;

impl MessageDispatcher {
    pub fn dispatch(raw_data: &[u8], model_info: &ModelInfo) -> Option<DispatchedPayload> {
        let key = protocol_handler::extract_key(model_info.protocol, raw_data)?;
        let payload = protocol_handler::unwrap_payload(model_info.protocol, key, raw_data);

        let result = match key {
            0x15 | 0x16 => DispatchedPayload::Alarms(decode_alarms_payload(payload)),
            0x18 => DispatchedPayload::Timer(decode_timer(payload)),
            0x23 => DispatchedPayload::WatchName(decode_watch_name(payload)),
            0x1E => DispatchedPayload::DstWorldCities(payload.to_vec()),
            0x1F => DispatchedPayload::WorldCities(payload.to_vec()),
            0x1D => DispatchedPayload::DstWatchState(payload.to_vec()),
            0x28 => DispatchedPayload::WatchCondition(decode_watch_condition(payload, 9, 19)),
            0x22 => DispatchedPayload::AppInfoResponse(prepare_app_info_response(payload)),
            0x10 => DispatchedPayload::ButtonPressed(decode_button_pressed(payload)),
            0x13 => DispatchedPayload::Settings(decode_settings(
                payload,
                LightDuration::TwoSeconds,
                LightDuration::FourSeconds,
            )),
            0x11 => DispatchedPayload::TimeAdjustment(decode_time_adjustment(payload)),
            0x26 => DispatchedPayload::StepCounter(StepCounterData::parse(payload)),
            0x24 => DispatchedPayload::HomeTime(parse_home_city(payload)),
            _ => DispatchedPayload::Unknown(key, payload.to_vec()),
        };

        Some(result)
    }

    pub fn action_request(action: &str) -> Vec<BleAction> {
        match action {
            "GET_ALARMS" => alarm_request(),
            "GET_SETTINGS" => vec![settings_request()],
            "GET_TIME_ADJUSTMENT" => vec![time_adjustment_request()],
            "GET_TIMER" => vec![timer_request()],
            "GET_HOME_TIME" => vec![home_time_request(0, false)],
            "GET_WATCH_CONDITION" => vec![watch_condition_request()],
            "GET_WATCH_NAME" => vec![watch_name_request()],
            "GET_APP_INFO" => vec![app_info_request()],
            "GET_BUTTON_PRESSED" => vec![button_request()],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_dispatcher_action_requests() {
        let alarm_reqs = MessageDispatcher::action_request("GET_ALARMS");
        assert_eq!(alarm_reqs.len(), 2);

        let timer_reqs = MessageDispatcher::action_request("GET_TIMER");
        assert_eq!(timer_reqs.len(), 1);
    }
}
