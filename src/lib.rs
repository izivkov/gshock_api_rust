//! Deterministic protocol primitives for Casio Bluetooth watches.
//!
//! This crate deliberately separates packet creation/parsing from BLE transport
//! so that protocol behavior can be tested without hardware.

pub mod api;
pub mod casio_constants;
pub mod casio_time_zone_helper;
pub mod connection;
pub mod iolib;
pub mod message_dispatcher;
pub mod model;
pub mod protocol_handler;
pub mod protocols;
pub mod watch_info;

// Root forwarders for backward compatibility
pub mod actions;
pub mod alarms;
pub mod app_info;
pub mod button;
pub mod dst_watch_state;
pub mod dst_world_cities;
pub mod gw_bx5600_time;
pub mod home_time;
pub mod protocol;
pub mod second_dial;
pub mod settings;
pub mod time;
pub mod time_adjustment;
pub mod time_zone;
pub mod timer;
pub mod watch;
pub mod watch_condition;
pub mod watch_name;
pub mod world_cities;

// Re-exports for convenience
pub use api::GshockAPI;
pub use casio_constants::*;
pub use casio_time_zone_helper::{CasioTimeZone, LatLon, find_time_zone, get_world_city_coordinates};
pub use connection::{CharacteristicNotification, Connection};
pub use iolib::*;
pub use message_dispatcher::{DispatchedPayload, MessageDispatcher};
pub use model::*;
pub use protocols::*;
pub use watch_info::{
    ModelInfo, ProtocolKind, WatchModel, derive_short_name, resolve_model, resolve_model_info,
};
