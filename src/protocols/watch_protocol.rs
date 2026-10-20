use crate::iolib::actions::BleAction;

pub trait WatchProtocol {
    fn extract_key(&self, data: &[u8]) -> Option<u8>;
    fn unwrap_payload<'a>(&self, key: u8, data: &'a [u8]) -> &'a [u8];
    fn get_watch_condition_request(&self) -> BleAction;
    fn get_timer_request(&self) -> BleAction;
}
