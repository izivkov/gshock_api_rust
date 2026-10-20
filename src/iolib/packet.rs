/// Casio command identifiers used by the protocol.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protocol {
    AppInfo = 0x22,
    WatchName = 0x23,
    BleFeatures = 0x10,
    SettingForBle = 0x11,
    AdvertiseParameterManager = 0x3b,
    ConnectionParameterManager = 0x3a,
    ModuleId = 0x26,
    WatchCondition = 0x28,
    VersionInformation = 0x20,
    DstWatchState = 0x1d,
    DstSetting = 0x1e,
    ServiceDiscoveryManager = 0x47,
    CurrentTime = 0x09,
    SettingForUserProfile = 0x45,
    SettingForTargetValue = 0x43,
    AlertLevel = 0x0a,
    SettingForAlm = 0x15,
    SettingForAlm2 = 0x16,
    SettingForBasic = 0x13,
    CurrentTimeManager = 0x39,
    WorldCities = 0x1f,
    ReminderTitle = 0x30,
    ReminderTime = 0x31,
    Timer = 0x18,
    Error = 0xff,
}

impl TryFrom<u8> for Protocol {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, ()> {
        use Protocol::*;
        match value {
            0x22 => Ok(AppInfo),
            0x23 => Ok(WatchName),
            0x10 => Ok(BleFeatures),
            0x11 => Ok(SettingForBle),
            0x3b => Ok(AdvertiseParameterManager),
            0x3a => Ok(ConnectionParameterManager),
            0x26 => Ok(ModuleId),
            0x28 => Ok(WatchCondition),
            0x20 => Ok(VersionInformation),
            0x1d => Ok(DstWatchState),
            0x1e => Ok(DstSetting),
            0x47 => Ok(ServiceDiscoveryManager),
            0x09 => Ok(CurrentTime),
            0x45 => Ok(SettingForUserProfile),
            0x43 => Ok(SettingForTargetValue),
            0x0a => Ok(AlertLevel),
            0x15 => Ok(SettingForAlm),
            0x16 => Ok(SettingForAlm2),
            0x13 => Ok(SettingForBasic),
            0x39 => Ok(CurrentTimeManager),
            0x1f => Ok(WorldCities),
            0x30 => Ok(ReminderTitle),
            0x31 => Ok(ReminderTime),
            0x18 => Ok(Timer),
            0xff => Ok(Error),
            _ => Err(()),
        }
    }
}
