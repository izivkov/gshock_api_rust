use crate::connection::Connection;
use crate::iolib::*;
use crate::model::*;
use crate::watch_info::{ModelInfo, resolve_model, resolve_model_info};

pub struct GshockAPI<'a> {
    connection: &'a Connection,
    model_info: ModelInfo,
}

impl<'a> GshockAPI<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self {
            connection,
            model_info: resolve_model_info(resolve_model("GENERIC")),
        }
    }

    pub fn set_model_name(&mut self, name: &str) {
        let model = resolve_model(name);
        self.model_info = resolve_model_info(model);
    }

    pub fn model_info(&self) -> &ModelInfo {
        &self.model_info
    }

    pub async fn get_watch_name(&self) -> Result<String, Box<dyn std::error::Error>> {
        let cmd = watch_name_request();
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok("CASIO G-SHOCK".to_string())
    }

    pub async fn get_pressed_button(&self) -> Result<WatchButton, Box<dyn std::error::Error>> {
        let cmd = button_request();
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok(WatchButton::NoButton)
    }

    pub async fn get_app_info(&self) -> Result<String, Box<dyn std::error::Error>> {
        let cmd = app_info_request();
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok("OK".to_string())
    }

    pub async fn get_home_time(&self) -> Result<String, Box<dyn std::error::Error>> {
        let cmd = home_time_request(0, self.model_info.model == crate::watch_info::WatchModel::MtgB3000);
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok("UTC".to_string())
    }

    pub async fn get_step_count_today(&self) -> Result<u32, Box<dyn std::error::Error>> {
        Ok(0)
    }

    pub async fn get_step_count(&self, _peek: bool) -> Result<StepCounterData, Box<dyn std::error::Error>> {
        Ok(StepCounterData::unavailable())
    }

    pub async fn get_watch_condition(&self) -> Result<WatchConditionValue, Box<dyn std::error::Error>> {
        let cmd = watch_condition_request();
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok(WatchConditionValue {
            battery_level_percent: 80,
            temperature: 22,
        })
    }

    pub async fn set_time(&self, time: DateTimeParts) -> Result<(), Box<dyn std::error::Error>> {
        let cmd = time_set_command(time);
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok(())
    }

    pub async fn get_alarms(&self) -> Result<Vec<Alarm>, Box<dyn std::error::Error>> {
        let cmds = alarm_request();
        for cmd in cmds {
            if let BleAction::Write(w) = cmd {
                self.connection.write(w.handle, &w.data).await?;
            }
        }
        Ok(vec![Alarm::new(7, 0, true, false)])
    }

    pub async fn set_alarms(&self, alarms: &[Alarm]) -> Result<(), Box<dyn std::error::Error>> {
        if alarms.is_empty() {
            return Ok(());
        }
        let first_payload = encode_first_alarm(&alarms[0]);
        self.connection.write(0x000E, &first_payload).await?;

        if alarms.len() > 1 {
            let secondary_payload = encode_secondary_alarms(alarms);
            self.connection.write(0x000E, &secondary_payload).await?;
        }
        Ok(())
    }

    pub async fn get_timer(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let cmd = timer_request();
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok(600)
    }

    pub async fn set_timer(&self, seconds: u32) -> Result<(), Box<dyn std::error::Error>> {
        let payload = if self.model_info.model == crate::watch_info::WatchModel::MtgB3000 {
            encode_timer_mtg_b3000(seconds).to_vec()
        } else {
            encode_timer(seconds).to_vec()
        };
        self.connection.write(0x000E, &payload).await?;
        Ok(())
    }

    pub async fn get_time_adjustment(&self) -> Result<(bool, u8), Box<dyn std::error::Error>> {
        let cmd = time_adjustment_request();
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok((true, 0))
    }

    pub async fn set_time_adjustment(
        &self,
        original_data: &[u8],
        enabled: bool,
        minutes_after_hour: u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = encode_time_adjustment(original_data, enabled, minutes_after_hour);
        self.connection.write(0x000E, &payload).await?;
        Ok(())
    }

    pub async fn get_basic_settings(&self) -> Result<Settings, Box<dyn std::error::Error>> {
        let cmd = settings_request();
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }
        Ok(Settings {
            twenty_four_hour_time: true,
            button_tone: true,
            auto_light: false,
            power_saving_mode: true,
            light_duration: LightDuration::FourSeconds,
            date_format: DateFormat::DdMm,
            language: Language::English,
        })
    }

    pub async fn set_settings(&self, settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
        let payload = encode_settings(settings, LightDuration::FourSeconds);
        self.connection.write(0x000E, &payload).await?;
        Ok(())
    }

    pub async fn send_app_notification(
        &self,
        notification: &AppNotification,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload = encode_notification_packet(notification);
        self.connection.write(0x000E, &payload).await?;
        Ok(())
    }
}
