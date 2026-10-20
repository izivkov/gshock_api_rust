/// Protocol family selected by a watch model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolKind {
    Standard,
    Mip,
    Analogue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WatchModel {
    Ga,
    Gw,
    DwB5600,
    Dw,
    Gmw,
    Gpr,
    Gst,
    Msg,
    Gb001,
    Gbd,
    Gbd800,
    MrgB5000,
    GcwB5000,
    Eqb,
    Ecb,
    Abl100,
    FB100,
    DwH5600,
    GmwBz5000,
    GwBx5600,
    MtgB1000,
    MtgB3000,
    Generic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelInfo {
    pub model: WatchModel,
    pub world_cities_count: u8,
    pub dst_count: u8,
    pub alarm_count: u8,
    pub has_auto_light: bool,
    pub has_reminders: bool,
    pub has_battery_level: bool,
    pub has_temperature: bool,
    pub always_connected: bool,
    pub has_step_counter: bool,
    pub has_date_format: bool,
    pub has_new_time_format: bool,
    pub has_second_dial: bool,
    pub has_hourly_chime: bool,
    pub settings_size: u8,
    pub protocol: ProtocolKind,
    pub has_home_time: bool,
    pub has_world_cities: bool,
    pub has_time_format: bool,
    pub has_power_saving_mode: bool,
    pub has_app_info: bool,
    pub chime_in_settings: bool,
    pub vibrate: bool,
}

const DEFAULT: ModelInfo = ModelInfo {
    model: WatchModel::Generic,
    world_cities_count: 2,
    dst_count: 1,
    alarm_count: 5,
    has_auto_light: false,
    has_reminders: false,
    has_battery_level: true,
    has_temperature: true,
    always_connected: false,
    has_step_counter: false,
    has_date_format: true,
    has_new_time_format: false,
    has_second_dial: false,
    has_hourly_chime: true,
    settings_size: 17,
    protocol: ProtocolKind::Standard,
    has_home_time: true,
    has_world_cities: true,
    has_time_format: true,
    has_power_saving_mode: true,
    has_app_info: true,
    chime_in_settings: false,
    vibrate: false,
};

/// Removes the `CASIO ` advertisement prefix and returns its first word.
pub fn derive_short_name(name: &str) -> &str {
    name.strip_prefix("CASIO ")
        .unwrap_or(name)
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("")
}

/// Resolves known advertised Casio names. Unknown names deliberately use the
/// generic capability profile, matching the Python reference implementation.
pub fn resolve_model(name: &str) -> WatchModel {
    let clean = name.strip_prefix("CASIO ").unwrap_or(name).trim();
    match clean {
        "GPR-B1000" | "GPR-H1000" => WatchModel::Gpr,
        "GMW-B5000" => WatchModel::Gmw,
        "GW-B5000" | "GW-B5600" | "GWR-B1000" | "GWG-B1000" | "GWR-B3000" => WatchModel::Gw,
        "MRG-B5000" => WatchModel::Gw,
        "GBD-800" | "GMD-B800" => WatchModel::Gbd800,
        "GBD-H1000" | "GBD-100" | "GBX-100" | "GBD-200" | "GBD-300" | "GBX-H5600" | "GDG-B100" => {
            WatchModel::Gbd
        }
        "DW-B5600" => WatchModel::DwB5600,
        "GBD-H2000" | "DW-GH5600" | "DW-H5600" | "GM-H5600" => WatchModel::DwH5600,
        "GMW-B5000#" | "GW-B5600#" | "MRG-B5000#" | "TRN-50" | "GCW-B5000" | "PRJ-BW002" => {
            WatchModel::GcwB5000
        }
        "ABL-100WE" | "ABL-100" => WatchModel::Abl100,
        "F-B100W" => WatchModel::FB100,
        "GMW-BZ5000" => WatchModel::GmwBz5000,
        "GW-BX5600" => WatchModel::GwBx5600,
        "ECB-800" | "ECB-10" | "ECB-500" | "ECB-20" | "ECB-30" | "ECB-40" | "ECB-S100"
        | "ECB-2000" | "ECB-2300" | "ECB-2200" | "ECB-S10" => WatchModel::Ecb,
        "ECB-900" | "ECB-950" | "GST-B200" | "GST-B300" | "GST-B400" | "GST-B1000" | "GST-B100"
        | "GST-B500" | "GST-B600" | "GST-W1000" => WatchModel::Gst,
        "EQB-1000" | "EQB-500" | "EQB-510" | "EQB-600" | "EQB-700" | "EQB-501" | "EQB-800"
        | "EQB-900" | "EQB-1100" | "EQB-1200" | "EQB-2000" | "EQB-1300" => WatchModel::Eqb,
        "MTG-B3000" | "MTG-B3100" => WatchModel::MtgB3000,
        "MTG-B1000" => WatchModel::MtgB1000,
        "GA-B2100" | "GM-B2100" | "GBM-2100" | "GA-B001" | "GA-B010" | "GBA-800" | "GBA-900"
        | "GBA-950" | "GBA-400" | "GB-5600A" | "GB-6900A" | "GB-5600B" | "GB-6900B"
        | "GB-X6900B" | "MRG-B2100" | "GMC-B2100" => WatchModel::Ga,
        "MSG-B100" => WatchModel::Msg,
        "G-B001" | "PRJ-B001" => WatchModel::Gb001,
        _ => WatchModel::Generic,
    }
}

pub fn resolve_model_info(model: WatchModel) -> ModelInfo {
    match model {
        WatchModel::Gw => ModelInfo {
            model,
            world_cities_count: 6,
            dst_count: 3,
            has_auto_light: true,
            has_reminders: true,
            ..DEFAULT
        },
        WatchModel::DwB5600 | WatchModel::Gmw | WatchModel::MrgB5000 | WatchModel::GcwB5000 => {
            ModelInfo {
                model,
                world_cities_count: 6,
                dst_count: 3,
                has_reminders: true,
                has_auto_light: matches!(
                    model,
                    WatchModel::Gmw | WatchModel::MrgB5000 | WatchModel::GcwB5000
                ),
                ..DEFAULT
            }
        }
        WatchModel::GmwBz5000 => ModelInfo {
            model,
            world_cities_count: 6,
            dst_count: 3,
            has_auto_light: true,
            ..DEFAULT
        },
        WatchModel::GwBx5600 => ModelInfo {
            model,
            world_cities_count: 6,
            dst_count: 3,
            has_auto_light: true,
            has_new_time_format: true,
            protocol: ProtocolKind::Mip,
            ..DEFAULT
        },
        WatchModel::MtgB1000 => ModelInfo {
            model,
            world_cities_count: 6,
            dst_count: 3,
            alarm_count: 1,
            has_auto_light: true,
            has_reminders: true,
            has_second_dial: true,
            has_hourly_chime: false,
            protocol: ProtocolKind::Analogue,
            ..DEFAULT
        },
        WatchModel::MtgB3000 => ModelInfo {
            model,
            alarm_count: 1,
            has_battery_level: true,
            has_auto_light: false,
            has_date_format: false,
            has_second_dial: true,
            has_hourly_chime: false,
            settings_size: 12,
            protocol: ProtocolKind::Analogue,
            has_time_format: false,
            has_power_saving_mode: false,
            ..DEFAULT
        },
        WatchModel::Abl100 | WatchModel::FB100 => ModelInfo {
            model,
            has_battery_level: false,
            has_temperature: false,
            has_step_counter: true,
            has_date_format: false,
            has_world_cities: false,
            has_home_time: false,
            ..DEFAULT
        },
        WatchModel::DwH5600 => ModelInfo {
            model,
            alarm_count: 4,
            has_auto_light: true,
            has_battery_level: false,
            always_connected: true,
            has_date_format: false,
            chime_in_settings: true,
            vibrate: true,
            ..DEFAULT
        },
        WatchModel::Gbd => ModelInfo {
            model,
            has_auto_light: true,
            has_temperature: false,
            has_world_cities: false,
            ..DEFAULT
        },
        WatchModel::Gbd800 => ModelInfo {
            model,
            has_auto_light: true,
            has_battery_level: false,
            has_temperature: false,
            has_world_cities: false,
            has_home_time: false,
            ..DEFAULT
        },
        WatchModel::Eqb => ModelInfo {
            model,
            has_auto_light: true,
            has_temperature: false,
            has_world_cities: false,
            ..DEFAULT
        },
        WatchModel::Ecb => ModelInfo {
            model,
            has_auto_light: true,
            has_battery_level: false,
            has_temperature: false,
            always_connected: true,
            has_power_saving_mode: false,
            ..DEFAULT
        },
        WatchModel::Gst | WatchModel::Ga | WatchModel::Msg => ModelInfo {
            model,
            has_reminders: true,
            ..DEFAULT
        },
        WatchModel::Gb001 => ModelInfo {
            model,
            has_auto_light: true,
            ..DEFAULT
        },
        WatchModel::Dw => ModelInfo {
            model,
            has_auto_light: true,
            ..DEFAULT
        },
        WatchModel::Gpr => ModelInfo {
            model,
            has_auto_light: true,
            ..DEFAULT
        },
        WatchModel::Generic => DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_reference_models_and_protocols() {
        let bx = resolve_model_info(resolve_model("CASIO GW-BX5600"));
        assert_eq!(bx.model, WatchModel::GwBx5600);
        assert!(bx.has_new_time_format);
        assert_eq!(bx.protocol, ProtocolKind::Mip);

        let mtg = resolve_model_info(resolve_model("CASIO MTG-B1000"));
        assert_eq!(mtg.model, WatchModel::MtgB1000);
        assert!(mtg.has_second_dial);
        assert_eq!(mtg.protocol, ProtocolKind::Analogue);
    }

    #[test]
    fn resolves_step_counter_models_and_generic_fallback() {
        let abl = resolve_model_info(resolve_model("CASIO ABL-100WE"));
        assert_eq!(abl.model, WatchModel::Abl100);
        assert!(abl.has_step_counter);
        assert_eq!(abl.protocol, ProtocolKind::Standard);
        assert_eq!(resolve_model("CASIO Not-A-Watch"), WatchModel::Generic);
    }

    #[test]
    fn derives_short_name_like_python_reference() {
        assert_eq!(derive_short_name("CASIO GW-B5600 3459"), "GW-B5600");
        assert_eq!(derive_short_name(" ECB-30"), "ECB-30");
    }
}
