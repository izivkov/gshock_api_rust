# gshock-api (Rust)

A Rust port of the Python [gshock_api](https://github.com/izivkov/gshock_api) project. It provides deterministic packet codecs, watch models, timezone helpers, and command generators for Casio Bluetooth watches. Protocol functionality can be used and tested without Bluetooth hardware; live BLE transport is available through an optional feature.

## Features

- BLE characteristic UUIDs, GATT handles, and command identifiers
- Watch model resolution and capability profiles
- Time packet encoding, timezone helpers, and world-city coordinates
- Alarm, reminder, step-counter, and app-notification models and codecs
- Settings, timer, watch-condition, watch-name, and button codecs
- World-city and daylight-saving-time codecs
- Time-adjustment commands and watch-specific time-setting sequences, including GW-BX5600 and GMW-BZ5000
- MTG-B1000 second-dial reset sequence

## Requirements

- Rust stable with Cargo (Rust edition 2024)
- For live BLE use: a Bluetooth adapter and operating-system BLE support. On Linux, BlueZ and the D-Bus development libraries required by `btleplug` must be installed.

## Install and Build

From the repository directory, build the crate:

```bash
cargo build
```

To use the crate from another local Rust project, add it as a path dependency in `Cargo.toml` (adjust the path to your clone):

```toml
[dependencies]
gshock-api = { path = "../gshock_api_rust" }
```

The default build does not require BLE dependencies. Enable live Bluetooth transport with the `ble` feature:

```toml
[dependencies]
gshock-api = { path = "../gshock_api_rust", features = ["ble"] }
```

## Usage

Protocol data can be parsed without connecting to a watch. For example, parse a step-counter payload:

```rust
use gshock_api::StepCounterData;

fn main() {
    let payload: &[u8] = &[/* bytes received from a watch */];

    match StepCounterData::parse(payload) {
        Some(data) => println!("Today's steps: {:?}", data.current_day_steps),
        None => println!("Payload was not a valid step-counter record"),
    }
}
```

The `examples/step_counter.rs` example builds a sample payload and exercises the parser:

```bash
cargo run --example step_counter
```

To run the time-setting server with real BLE transport, enable the feature:

```bash
cargo run --features ble --example gshock_time_server
```

The server scans for a compatible Casio watch and sets its time when triggered from the watch. The `gshock_server_bx` example demonstrates the GW-BX5600 / GMW-BZ5000 time-setting sequence:

```bash
cargo run --features ble --example gshock_server_bx
```

Without `--features ble`, `Connection` uses its simulated transport, which is useful for exercising command flow without a Bluetooth adapter.

## Tests

Run the full test suite with:

```bash
cargo test
```

## Relationship to the Python Project

This crate is based on the Python [gshock_api](https://github.com/izivkov/gshock_api) library. The Python project supports the broader watch API; this Rust crate ports protocol and model functionality into Rust and is developed independently. Consult the Python project for its own installation instructions, supported models, and API documentation.
