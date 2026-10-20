/// A side-effect-free command for a future BLE transport adapter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BleAction {
    Write(Write),
    Read { handle: u16 },
}

/// A BLE characteristic write.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Write {
    pub handle: u16,
    pub data: Vec<u8>,
}

impl Write {
    pub fn new(handle: u16, data: impl Into<Vec<u8>>) -> Self {
        Self {
            handle,
            data: data.into(),
        }
    }
}
