# gshock-api (Rust)

A pure, deterministic Rust port of the Python `gshock_api` library. This crate provides protocol packet codecs, domain models, timezone helpers, command stream generators, and BLE transport for Casio Bluetooth watches.

## Features & Capabilities

- **Real BLE Transport & Scanner**: Connected to `btleplug` by default (`features = ["ble"]`), supporting real hardware connections over Bluetooth LE on Linux, macOS, and Windows.
- **Protocol & GATT Handles**: Complete BLE characteristic UUIDs and GATT handle mappings (`0x0C`, `0x0E`, `0x11`, `0x14`, `0x17`, `0x19`, etc.) matching the Python reference implementation.
- **Watch Model Resolution & Profiles**: Full `WatchModel` resolution and capability profiles (`ModelInfo`).
- **Time & Timezone Helpers**: Current time encoding, `CasioTimeZone`, and world city coordinate lookup.
- **Domain Models**:
  - `Alarm`: Standard & MTG-B3000 alarm encoding/decoding.
  - `Event` / `EventDate` / `RepeatPeriod`: Reminder titles and time detail codecs.
  - `StepCounterData`: ABL-100WE life-log record parsing and BCD decode.
  - `AppNotification`: Phone calendar/message notification packet codec with XOR encoding/decoding and UTF-8 length enforcement.
- **Stateless Codecs & Response Handlers**:
  - Basic & MTG-B3000 settings codecs
  - Timer codecs
  - AppInfo request/response
  - Watch condition (battery level & temperature)
  - Watch name & button press decoder
  - World cities, DST watch state, and DST world cities
  - Time adjustment (auto-sync setting)
  - GW-BX5600 / GMW-BZ5000 4-step time set sequence & 22-byte city location records
  - MTG-B1000 second dial reset sequence

## Requirements (Linux)

When building on Linux with real Bluetooth LE hardware transport (default):
- `libdbus-1-dev` and `pkg-config`

```bash
# Ubuntu / Debian
sudo apt install -y libdbus-1-dev pkg-config

# Fedora / RHEL
sudo dnf install -y dbus-devel pkgconf-pkg-config
```

## Running Examples

See the [API documentation](doc/API.md) for connection setup, the
`GshockAPI` methods, return values, and fallback behavior.

Execute real tests and time server examples:

```bash
# Run API test suite
cargo run --example api_tests

# Run time server
cargo run --example gshock_time_server -- --fine-adjustment-secs 0

# Run GW-BX5600 MIP display time server
cargo run --example gshock_server_bx

# Run step counter example
cargo run --example step_counter

# Run app notification codec test
cargo run --example app_notifications_tests
```

## Running Unit & Integration Tests

```bash
cargo test
```
