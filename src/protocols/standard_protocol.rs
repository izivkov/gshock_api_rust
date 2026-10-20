use super::watch_protocol::WatchProtocol;
use crate::iolib::actions::BleAction;
use crate::iolib::timer_io::timer_request;
use crate::iolib::watch_condition_io::watch_condition_request;

pub struct StandardProtocol;

impl WatchProtocol for StandardProtocol {
    fn extract_key(&self, data: &[u8]) -> Option<u8> {
        if data.is_empty() {
            return None;
        }
        let key = data[0];
        if key == 0 { None } else { Some(key) }
    }

    fn unwrap_payload<'a>(&self, _key: u8, data: &'a [u8]) -> &'a [u8] {
        data
    }

    fn get_watch_condition_request(&self) -> BleAction {
        watch_condition_request()
    }

    fn get_timer_request(&self) -> BleAction {
        timer_request()
    }
}
