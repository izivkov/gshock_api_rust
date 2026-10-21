# G-Shock API

This document describes the crate's high-level Rust API, exported as
`gshock_api::GshockAPI`. The exported facade is currently implemented in
`src/api.rs` and re-exported by `src/lib.rs`.

## Requirements

The default `ble` feature uses `btleplug` for Bluetooth Low Energy. On Linux,
install the system packages listed in the [README](../README.md#requirements-linux).
Use `--no-default-features` to build without real BLE transport; in that mode,
connections and writes are simulated.

## Connect and use

```rust,no_run
use gshock_api::{Connection, GshockAPI};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut connection = Connection::new();
    if !connection.connect().await? {
        return Err("No Bluetooth adapter found".into());
    }

    let mut watch = GshockAPI::new(&connection);
    let name = watch.get_watch_name().await?;
    let condition = watch.get_watch_condition().await?;
    println!("{name}: {}% battery", condition.battery_level_percent);

    connection.disconnect().await?;
    Ok(())
}
```

With BLE enabled, `Connection::connect` scans until a supported Casio watch is
found and connected. It returns `false` when no Bluetooth adapter is available.
Keep the `Connection` alive for at least as long as the `GshockAPI` borrowing it.
`GshockAPI::new` starts with a generic model profile; `get_watch_name` detects
the advertised model name and updates that profile. You can also set it
explicitly with `set_model_name`.

## `GshockAPI`

All methods that communicate with the watch are asynchronous and return
`Result<_, Box<dyn std::error::Error>>`. Read operations wait up to three
seconds for a matching notification. If no response arrives, most return a
documented fallback value, so a successful `Result` does not necessarily mean
the watch supplied the value.

### Model information

| Method | Description |
| --- | --- |
| `new(&Connection) -> GshockAPI` | Create an API client using the generic capability profile. |
| `set_model_name(&mut self, name: &str)` | Resolve a known watch name and select its capability profile. Unknown names use the generic profile. |
| `model_info(&self) -> &ModelInfo` | Inspect the current profile, including feature flags and supported counts. |

### Watch information and status

| Method | Result and behavior |
| --- | --- |
| `get_watch_name(&mut self)` | Watch name; updates the model profile. Falls back to `"CASIO G-SHOCK"`. |
| `get_pressed_button(&self)` | `WatchButton`; falls back to `WatchButton::NoButton`. |
| `get_app_info(&self)` | Returns `"OK"` when supported and acknowledged; also returns `"OK"` when unsupported or no response arrives. |
| `get_home_time(&self)` | Home-city name; falls back to `"UTC"`. |
| `get_watch_condition(&self)` | `WatchConditionValue` with battery percentage and temperature; falls back to 80% and 22 degrees. |
| `get_step_count_today(&self)` | Today's step count, or `0` when the watch has no supported counter or no data is available. |
| `get_step_count(&self, peek: bool)` | `StepCounterData`; unavailable models or missing responses return `StepCounterData::unavailable()`. The `peek` argument is currently ignored. |
| `get_dst_watch_state_raw(&self, state: u8)` | Raw DST state bytes; returns a fallback payload on timeout. |
| `get_dst_for_world_cities_raw(&self, city_num: u8)` | Raw DST data for a city; returns a fallback payload on timeout. |
| `get_world_cities_raw(&self, city_num: u8)` | Raw city data; returns a UTC city payload on timeout. |

### Time

`DateTimeParts` contains `year`, `month`, `day`, `hour`, `minute`, `second`,
`weekday`, and `nanosecond`. `weekday` follows Python's weekday numbering:
Monday is `0`, Sunday is `6`.

| Method | Description |
| --- | --- |
| `initialize_for_setting_time(&self)` | Synchronize the watch's DST and world-city configuration before time setting. |
| `set_time(&self, time: Option<DateTimeParts>, offset_secs: i64)` | Set the supplied time, or use local system time adjusted by `offset_secs`. Also initializes watch time settings and model-specific second-dial state. |
| `set_system_time(&self, offset_secs: i64)` | Convenience wrapper that sets local system time adjusted by `offset_secs`. |

For example, pass `None` and `0` to set the current local system time. A positive
offset adds seconds; a negative offset subtracts them.

### Alarms, timer, and time adjustment

`Alarm` has public `hour`, `minute`, `enabled`, and `has_hourly_chime` fields and
can be constructed with `Alarm::new(hour, minute, enabled, has_hourly_chime)`.

| Method | Description |
| --- | --- |
| `get_alarms(&self)` | Read alarms; falls back to one enabled alarm at 07:00 if no alarm response arrives. |
| `set_alarms(&self, alarms: &[Alarm])` | Write the first and, when provided, secondary alarms. An empty slice does nothing. |
| `get_timer(&self)` | Timer duration in seconds; falls back to `600`. |
| `set_timer(&self, seconds: u32)` | Set the timer duration in seconds, using the MTG-B3000 encoding when applicable. |
| `get_time_adjustment(&self)` | Returns `(enabled, minutes_after_hour)`; falls back to `(true, 0)`. |
| `set_time_adjustment(&self, original_data: &[u8], enabled: bool, minutes_after_hour: u8)` | Update time adjustment while preserving fields from the original response payload. |

### Basic settings and notifications

`Settings` contains `twenty_four_hour_time`, `button_tone`, `auto_light`,
`power_saving_mode`, `light_duration`, `date_format`, and `language`. The enum
options are `LightDuration`, `DateFormat`, and `Language` from `gshock_api`.

| Method | Description |
| --- | --- |
| `get_basic_settings(&self)` | Read basic watch settings; returns a default settings value if no response arrives. |
| `set_settings(&self, settings: Settings)` | Write the supplied basic settings. |
| `send_app_notification(&self, notification: &AppNotification)` | Send a notification packet to the watch. Construct notifications with `AppNotification::new`; its text fields are truncated to protocol byte limits on UTF-8 boundaries. |

`AppNotification` takes a `NotificationType`, timestamp, app name, title, full
text, and short text. Notification types include generic, phone call, email,
message, and calendar notifications.

## Related public types

The crate root re-exports the high-level data types, including `Alarm`,
`AppNotification`, `DateTimeParts`, `ModelInfo`, `Settings`, `StepCounterData`,
`WatchButton`, `WatchConditionValue`, and model capability enums. See the Rust
API documentation generated by `cargo doc --open` for full field-level and
codec documentation.