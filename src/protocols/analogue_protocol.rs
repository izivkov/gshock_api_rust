use super::watch_protocol::WatchProtocol;
use crate::iolib::actions::BleAction;
use crate::iolib::timer_io::timer_request;
use crate::iolib::watch_condition_io::watch_condition_request;

pub struct AnalogueProtocol;

impl WatchProtocol for AnalogueProtocol {
    fn extract_key(&self, data: &[u8]) -> Option<u8> {
        if data.is_empty() {
            return None;
        }
        let first_byte = data[0];
        if first_byte == 0x28 && data.len() > 4 {
            if data[1] == 0x01 {
                Some(data[4])
            } else if data[1] == 0x00 {
                Some(data[3])
            } else {
                Some(0x28)
            }
        } else {
            Some(first_byte)
        }
    }

    fn unwrap_payload<'a>(&self, key: u8, data: &'a [u8]) -> &'a [u8] {
        if data.is_empty() {
            return data;
        }
        if data[0] == 0x28 && key != 0x28 {
            let skip = if data.len() > 1 && data[1] == 0x01 { 4 } else { 3 };
            if data.len() >= skip {
                &data[skip..]
            } else {
                data
            }
        } else {
            data
        }
    }

    fn get_watch_condition_request(&self) -> BleAction {
        watch_condition_request()
    }

    fn get_timer_request(&self) -> BleAction {
        timer_request()
    }
}
