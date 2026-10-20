use super::standard_protocol::StandardProtocol;
use super::watch_protocol::WatchProtocol;
use crate::iolib::actions::BleAction;

pub struct MipProtocol;

impl WatchProtocol for MipProtocol {
    fn extract_key(&self, data: &[u8]) -> Option<u8> {
        StandardProtocol.extract_key(data)
    }

    fn unwrap_payload<'a>(&self, key: u8, data: &'a [u8]) -> &'a [u8] {
        StandardProtocol.unwrap_payload(key, data)
    }

    fn get_watch_condition_request(&self) -> BleAction {
        StandardProtocol.get_watch_condition_request()
    }

    fn get_timer_request(&self) -> BleAction {
        StandardProtocol.get_timer_request()
    }
}
