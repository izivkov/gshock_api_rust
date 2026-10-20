#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightDuration {
    OnePointFiveSeconds,
    TwoSeconds,
    ThreeSeconds,
    FourSeconds,
    FiveSeconds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateFormat {
    DdMm,
    MmDd,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    Spanish,
    French,
    German,
    Italian,
    Russian,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub twenty_four_hour_time: bool,
    pub button_tone: bool,
    pub auto_light: bool,
    pub power_saving_mode: bool,
    pub light_duration: LightDuration,
    pub date_format: DateFormat,
    pub language: Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BasicSettings {
    pub button_tone: bool,
    pub power_saving_mode: bool,
    pub light_duration: LightDuration,
}
