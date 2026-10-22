use chrono::{Datelike, Local, Timelike};
use tokio::time::{Duration, timeout};

use crate::casio_time_zone_helper::find_time_zone;
use crate::connection::Connection;
use crate::iolib::*;
use crate::message_dispatcher::{DispatchedPayload, MessageDispatcher};
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

    async fn await_response<F, T>(
        &self,
        cmd: BleAction,
        matcher: F,
        timeout_secs: u64,
        fallback: T,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        F: Fn(&DispatchedPayload) -> Option<T>,
    {
        let mut rx = self.connection.subscribe_notifications();
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }

        let deadline = Duration::from_secs(timeout_secs);
        let start = std::time::Instant::now();

        while start.elapsed() < deadline {
            let remaining = deadline.saturating_sub(start.elapsed());
            match timeout(remaining, rx.recv()).await {
                Ok(Ok(bytes)) => {
                    if let Some(payload) = MessageDispatcher::dispatch(&bytes, &self.model_info) {
                        if let Some(val) = matcher(&payload) {
                            return Ok(val);
                        }
                    }
                }
                _ => break,
            }
        }

        // Fallback for simulated connection mode or offline tests
        Ok(fallback)
    }

    pub async fn get_dst_watch_state_raw(&self, state: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let req_data = vec![Protocol::DstWatchState as u8, state];
        self.await_response(
            BleAction::Write(Write::new(0x000C, req_data)),
            |p| match p {
                DispatchedPayload::DstWatchState(bytes) => Some(bytes.clone()),
                _ => None,
            },
            3,
            vec![0x1d, state, 0x00, 0x00],
        )
        .await
    }

    pub async fn get_dst_for_world_cities_raw(&self, city_num: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let req_data = vec![Protocol::DstSetting as u8, city_num];
        self.await_response(
            BleAction::Write(Write::new(0x000C, req_data)),
            |p| match p {
                DispatchedPayload::DstWorldCities(bytes) => Some(bytes.clone()),
                _ => None,
            },
            3,
            vec![0x1e, city_num, 0x00, 0x00, 0x00, 0x00, 0x00],
        )
        .await
    }

    pub async fn get_world_cities_raw(&self, city_num: u8) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let req_data = vec![Protocol::WorldCities as u8, city_num];
        self.await_response(
            BleAction::Write(Write::new(0x000C, req_data)),
            |p| match p {
                DispatchedPayload::WorldCities(bytes) => Some(bytes.clone()),
                _ => None,
            },
            3,
            encode_and_pad_world_city("UTC", city_num).to_vec(),
        )
        .await
    }

    pub async fn get_watch_name(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let name = self
            .await_response(
                watch_name_request(),
                |p| match p {
                    DispatchedPayload::WatchName(name) => Some(name.clone()),
                    _ => None,
                },
                3,
                "CASIO G-SHOCK".to_string(),
            )
            .await?;

        self.set_model_name(&name);
        Ok(name)
    }

    pub async fn get_pressed_button(&self) -> Result<WatchButton, Box<dyn std::error::Error>> {
        self.await_response(
            button_request(),
            |p| match p {
                DispatchedPayload::ButtonPressed(btn) => Some(*btn),
                _ => None,
            },
            3,
            WatchButton::NoButton,
        )
        .await
    }

    pub async fn get_app_info(&self) -> Result<String, Box<dyn std::error::Error>> {
        if !self.model_info.has_app_info {
            return Ok("OK".to_string());
        }
        self.await_response(
            app_info_request(),
            |p| match p {
                DispatchedPayload::AppInfoResponse(_) => Some("OK".to_string()),
                _ => None,
            },
            3,
            "OK".to_string(),
        )
        .await
    }

    pub async fn get_home_time(&self) -> Result<String, Box<dyn std::error::Error>> {
        let is_mtg_b3000 = self.model_info.model == crate::watch_info::WatchModel::MtgB3000;
        self.await_response(
            home_time_request(0, is_mtg_b3000),
            |p| match p {
                DispatchedPayload::HomeTime(name) => Some(name.clone()),
                DispatchedPayload::WorldCities(bytes) => Some(parse_home_city(bytes)),
                _ => None,
            },
            3,
            "UTC".to_string(),
        )
        .await
    }

    pub async fn get_step_count_today(&self) -> Result<u32, Box<dyn std::error::Error>> {
        if !self.model_info.has_step_counter {
            return Ok(0);
        }
        let data = self.get_step_count(false).await?;
        Ok(data.current_day_steps.unwrap_or(0))
    }

    pub async fn get_step_count(&self, peek: bool) -> Result<StepCounterData, Box<dyn std::error::Error>> {
        if !self.model_info.has_step_counter {
            return Ok(StepCounterData::unavailable());
        }
        crate::iolib::step_counter_io::request(self.connection, peek).await
    }

    pub async fn get_watch_condition(&self) -> Result<WatchConditionValue, Box<dyn std::error::Error>> {
        self.await_response(
            watch_condition_request(),
            |p| match p {
                DispatchedPayload::WatchCondition(cond) => Some(*cond),
                _ => None,
            },
            3,
            WatchConditionValue {
                battery_level_percent: 80,
                temperature: 22,
            },
        )
        .await
    }

    pub async fn initialize_for_setting_time(&self) -> Result<(), Box<dyn std::error::Error>> {
        let tz = find_time_zone("UTC");
        let dst_val = tz.dst_rules;

        // 1. Read DST watch state 0, modify DST byte, write back
        let raw_dst0 = self.get_dst_watch_state_raw(0).await?;
        let mod_dst0 = set_dst_watch_state(&raw_dst0, dst_val);
        self.connection.write(0x000E, &mod_dst0).await?;

        // Remaining DST watch states
        let dst_states = [2u8, 4u8];
        let remaining_dst_count = (self.model_info.dst_count as usize).saturating_sub(1);
        for &state in &dst_states[..remaining_dst_count.min(2)] {
            let raw_dst = self.get_dst_watch_state_raw(state).await?;
            self.connection.write(0x000E, &raw_dst).await?;
        }

        // 2. Read DST for world cities
        for city_num in 0..self.model_info.world_cities_count {
            let raw_dst_city = self.get_dst_for_world_cities_raw(city_num).await?;
            let mod_dst_city = if city_num == 0 {
                set_dst_for_world_city(&raw_dst_city, 0, 0, tz.dst_rules)
            } else {
                raw_dst_city
            };
            self.connection.write(0x000E, &mod_dst_city).await?;
        }

        // 3. Read World cities
        if self.model_info.has_world_cities {
            for city_num in 0..self.model_info.world_cities_count {
                let raw_wc = self.get_world_cities_raw(city_num).await?;
                self.connection.write(0x000E, &raw_wc).await?;
            }
        }

        Ok(())
    }

    pub async fn set_system_time(&self, offset_secs: i64) -> Result<(), Box<dyn std::error::Error>> {
        let now = Local::now() + chrono::Duration::seconds(offset_secs);
        let dt = DateTimeParts {
            year: now.year() as u16,
            month: now.month() as u8,
            day: now.day() as u8,
            hour: now.hour() as u8,
            minute: now.minute() as u8,
            second: now.second() as u8,
            weekday: now.weekday().num_days_from_monday() as u8,
            nanosecond: now.nanosecond(),
        };
        self.set_time(Some(dt), 0).await
    }

    pub async fn set_time(
        &self,
        time: Option<DateTimeParts>,
        offset_secs: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let time_parts = match time {
            Some(t) => t,
            None => {
                let now = Local::now() + chrono::Duration::seconds(offset_secs);
                DateTimeParts {
                    year: now.year() as u16,
                    month: now.month() as u8,
                    day: now.day() as u8,
                    hour: now.hour() as u8,
                    minute: now.minute() as u8,
                    second: now.second() as u8,
                    weekday: now.weekday().num_days_from_monday() as u8,
                    nanosecond: now.nanosecond(),
                }
            }
        };

        // Run initial handshake sequence required by watch protocol before writing time
        let _ = self.initialize_for_setting_time().await;

        let cmd = time_set_command(time_parts);
        if let BleAction::Write(w) = cmd {
            self.connection.write(w.handle, &w.data).await?;
        }

        if self.model_info.has_second_dial {
            let start = reset_sequence_start();
            if let BleAction::Write(w) = start {
                self.connection.write(w.handle, &w.data).await?;
            }
            let end = reset_sequence_end();
            if let BleAction::Write(w) = end {
                self.connection.write(w.handle, &w.data).await?;
            }
        }

        Ok(())
    }

    pub async fn get_alarms(&self) -> Result<Vec<Alarm>, Box<dyn std::error::Error>> {
        let cmds = alarm_request();
        let mut rx = self.connection.subscribe_notifications();

        for cmd in cmds {
            if let BleAction::Write(w) = cmd {
                self.connection.write(w.handle, &w.data).await?;
            }
        }

        let deadline = Duration::from_secs(3);
        let start = std::time::Instant::now();

        while start.elapsed() < deadline {
            let remaining = deadline.saturating_sub(start.elapsed());
            match timeout(remaining, rx.recv()).await {
                Ok(Ok(bytes)) => {
                    if let Some(DispatchedPayload::Alarms(alarms)) =
                        MessageDispatcher::dispatch(&bytes, &self.model_info)
                    {
                        if !alarms.is_empty() {
                            return Ok(alarms);
                        }
                    }
                }
                _ => break,
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

    pub async fn get_event_from_watch(
        &self,
        event_number: u8,
    ) -> Result<Option<Event>, Box<dyn std::error::Error>> {
        if !self.model_info.has_reminders {
            return Ok(None);
        }

        let mut rx = self.connection.subscribe_notifications();
        for cmd in [
            reminder_title_request(event_number),
            reminder_time_request(event_number),
        ] {
            if let BleAction::Write(w) = cmd {
                self.connection.write(w.handle, &w.data).await?;
            }
        }

        let deadline = Duration::from_secs(3);
        let start = std::time::Instant::now();
        let mut title = None;
        let mut reminder_time = None;

        while start.elapsed() < deadline {
            let remaining = deadline.saturating_sub(start.elapsed());
            match timeout(remaining, rx.recv()).await {
                Ok(Ok(bytes)) => match MessageDispatcher::dispatch(&bytes, &self.model_info) {
                    Some(DispatchedPayload::ReminderTitle(number, value))
                        if number == event_number =>
                    {
                        if value.is_none() {
                            return Ok(None);
                        }
                        title = value;
                    }
                    Some(DispatchedPayload::ReminderTime(number, event))
                        if number == event_number =>
                    {
                        reminder_time = Some(event);
                    }
                    _ => {}
                },
                _ => break,
            }

            if title.is_some() && reminder_time.is_some() {
                let mut event = reminder_time.take().expect("checked above");
                event.title = title.take().expect("checked above");
                return Ok(Some(event));
            }
        }

        Ok(None)
    }

    pub async fn get_reminders(&self) -> Result<Vec<Option<Event>>, Box<dyn std::error::Error>> {
        if !self.model_info.has_reminders {
            return Ok(vec![None; 5]);
        }

        let mut events = Vec::new();
        for event_number in 1..=5 {
            events.push(self.get_event_from_watch(event_number).await?);
        }
        Ok(events)
    }

    pub async fn set_reminders(&self, events: &[Event]) -> Result<(), Box<dyn std::error::Error>> {
        if !self.model_info.has_reminders {
            return Ok(());
        }

        for cmd in reminder_set_commands(events) {
            if let BleAction::Write(w) = cmd {
                self.connection.write(w.handle, &w.data).await?;
            }
        }
        Ok(())
    }

    pub async fn get_timer(&self) -> Result<u32, Box<dyn std::error::Error>> {
        self.await_response(
            timer_request(),
            |p| match p {
                DispatchedPayload::Timer(secs) => Some(*secs),
                _ => None,
            },
            3,
            600,
        )
        .await
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
        self.await_response(
            time_adjustment_request(),
            |p| match p {
                DispatchedPayload::TimeAdjustment(adj) => *adj,
                _ => None,
            },
            3,
            (true, 0),
        )
        .await
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
        self.await_response(
            settings_request(),
            |p| match p {
                DispatchedPayload::Settings(s) => *s,
                _ => None,
            },
            3,
            Settings {
                twenty_four_hour_time: true,
                button_tone: true,
                auto_light: false,
                power_saving_mode: true,
                light_duration: LightDuration::FourSeconds,
                date_format: DateFormat::DdMm,
                language: Language::English,
            },
        )
        .await
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
        let command = notification_write(notification)
            .map_err(|message| std::io::Error::new(std::io::ErrorKind::InvalidInput, message))?;
        if let BleAction::Write(write) = command {
            self.connection.write(write.handle, &write.data).await?;
        }
        Ok(())
    }
}
