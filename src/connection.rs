use tokio::sync::broadcast;

#[cfg(feature = "ble")]
use btleplug::api::{Central, CharPropFlags, Manager as _, Peripheral as _, ScanFilter, WriteType};
#[cfg(feature = "ble")]
use btleplug::platform::{Adapter, Manager, Peripheral};
#[cfg(feature = "ble")]
use futures_util::StreamExt;
#[cfg(feature = "ble")]
use std::time::Duration;
#[cfg(feature = "ble")]
use uuid::Uuid;

#[cfg(feature = "ble")]
use crate::casio_constants::*;

pub const CASIO_SERVICE_UUID: &str = "00001804-0000-1000-8000-00805f9b34fb";

/// Set to `false` to easily suppress sent/received BLE screen logs.
pub const LOG_SENT_RECV: bool = true;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacteristicNotification {
    pub handle: u16,
    pub data: Vec<u8>,
}

fn log_sent(handle: u16, data: &[u8]) {
    if LOG_SENT_RECV {
        println!("[BLE SENT] Handle 0x{:04X} -> {}", handle, hex::encode(data));
    }
}

fn log_recv(data: &[u8]) {
    if LOG_SENT_RECV {
        println!("[BLE RECV] <- {}", hex::encode(data));
    }
}

pub struct Connection {
    #[cfg(feature = "ble")]
    adapter: Option<Adapter>,
    #[cfg(feature = "ble")]
    peripheral: Option<Peripheral>,
    notification_tx: broadcast::Sender<Vec<u8>>,
    characteristic_notification_tx: broadcast::Sender<CharacteristicNotification>,
}

impl Connection {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        let (characteristic_tx, _characteristic_rx) = broadcast::channel(100);
        Self {
            #[cfg(feature = "ble")]
            adapter: None,
            #[cfg(feature = "ble")]
            peripheral: None,
            notification_tx: tx,
            characteristic_notification_tx: characteristic_tx,
        }
    }

    pub fn subscribe_notifications(&self) -> broadcast::Receiver<Vec<u8>> {
        self.notification_tx.subscribe()
    }

    pub fn subscribe_characteristic_notifications(
        &self,
    ) -> broadcast::Receiver<CharacteristicNotification> {
        self.characteristic_notification_tx.subscribe()
    }

    pub fn send_notification(&self, data: Vec<u8>) {
        log_recv(&data);
        let _ = self.notification_tx.send(data);
    }

    pub fn send_characteristic_notification(&self, handle: u16, data: Vec<u8>) {
        log_recv(&data);
        let _ = self.notification_tx.send(data.clone());
        let _ = self
            .characteristic_notification_tx
            .send(CharacteristicNotification { handle, data });
    }

    #[cfg(feature = "ble")]
    pub async fn connect(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let manager = Manager::new().await?;
        let adapters = manager.adapters().await?;
        if adapters.is_empty() {
            println!("No Bluetooth adapters found.");
            return Ok(false);
        }
        let adapter = adapters[0].clone();
        let casio_service = Uuid::parse_str(CASIO_SERVICE_UUID)?;

        loop {
            let _ = adapter.start_scan(ScanFilter::default()).await;
            tokio::time::sleep(Duration::from_secs(2)).await;

            let peripherals = match adapter.peripherals().await {
                Ok(p) => p,
                Err(_) => continue,
            };

            for peripheral in peripherals {
                if let Ok(Some(props)) = peripheral.properties().await {
                    let name = props.local_name.unwrap_or_default();
                    let has_casio_service = props.services.iter().any(|uuid| *uuid == casio_service);
                    let is_casio_name = name.trim().starts_with("CASIO");

                    if has_casio_service || (is_casio_name && !props.services.is_empty()) {
                        if peripheral.connect().await.is_ok() {
                            println!("Found and connected to Casio watch: {}", name);
                            let _ = peripheral.discover_services().await;

                            // Subscribe to notify/indicate characteristics
                            for char in peripheral.characteristics() {
                                if char.properties.contains(CharPropFlags::NOTIFY)
                                    || char.properties.contains(CharPropFlags::INDICATE)
                                {
                                    let _ = peripheral.subscribe(&char).await;
                                }
                            }

                            // Spawn notification listener task
                            if let Ok(mut stream) = peripheral.notifications().await {
                                let tx = self.notification_tx.clone();
                                let characteristic_tx = self.characteristic_notification_tx.clone();
                                tokio::spawn(async move {
                                    while let Some(notif) = stream.next().await {
                                        log_recv(&notif.value);
                                        let handle = match notif.uuid.to_string().as_str() {
                                            CASIO_DATA_REQUEST_SP_CHARACTERISTIC_UUID => 0x0011,
                                            CASIO_CONVOY_CHARACTERISTIC_UUID => 0x0014,
                                            CASIO_ALL_FEATURES_CHARACTERISTIC_UUID => 0x000e,
                                            CASIO_NOTIFICATION_CHARACTERISTIC_UUID => 0x000d,
                                            _ => 0,
                                        };
                                        let _ = tx.send(notif.value.clone());
                                        let _ = characteristic_tx.send(CharacteristicNotification {
                                            handle,
                                            data: notif.value,
                                        });
                                    }
                                });
                            }

                            self.adapter = Some(adapter);
                            self.peripheral = Some(peripheral);
                            return Ok(true);
                        }
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    #[cfg(not(feature = "ble"))]
    pub async fn connect(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        println!("Simulated Connection: connected to Casio watch mock.");
        Ok(true)
    }

    #[cfg(feature = "ble")]
    pub async fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref peripheral) = self.peripheral {
            peripheral.disconnect().await?;
        }
        Ok(())
    }

    #[cfg(not(feature = "ble"))]
    pub async fn disconnect(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Simulated Connection: disconnected.");
        Ok(())
    }

    #[cfg(feature = "ble")]
    pub async fn write(&self, handle: u16, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        log_sent(handle, data);
        if let Some(ref peripheral) = self.peripheral {
            let uuid_str = match handle {
                0x04 => CASIO_GET_DEVICE_NAME,
                0x06 => CASIO_APPEARANCE,
                0x09 => TX_POWER_LEVEL_CHARACTERISTIC_UUID,
                0x0C => CASIO_READ_REQUEST_FOR_ALL_FEATURES_CHARACTERISTIC_UUID,
                0x0D => CASIO_NOTIFICATION_CHARACTERISTIC_UUID,
                0x0E => CASIO_ALL_FEATURES_CHARACTERISTIC_UUID,
                0x11 => CASIO_DATA_REQUEST_SP_CHARACTERISTIC_UUID,
                0x14 => CASIO_CONVOY_CHARACTERISTIC_UUID,
                0x17 => CASIO_SET_CONFIGURATION_CHARACTERISTIC_UUID,
                0x19 => CASIO_GET_CONFIGURATION_CHARACTERISTIC_UUID,
                _ => CASIO_ALL_FEATURES_CHARACTERISTIC_UUID,
            };

            let char_uuid = Uuid::parse_str(uuid_str)?;
            let write_type = match handle {
                0x0C | 0x0D | 0x14 | 0x17 => WriteType::WithoutResponse,
                _ => WriteType::WithResponse,
            };

            if let Some(characteristic) = peripheral.characteristics().iter().find(|c| c.uuid == char_uuid) {
                if let Err(e) = peripheral.write(characteristic, data, write_type).await {
                    println!("[BLE WRITE WARN] Handle 0x{:04X} write error: {}, ignoring.", handle, e);
                }
            } else {
                println!("[BLE WRITE WARN] Characteristic for handle 0x{:04X} not found on watch.", handle);
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "ble"))]
    pub async fn write(&self, handle: u16, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        log_sent(handle, data);
        Ok(())
    }

    pub async fn request(&self, command_hex: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(bytes) = hex::decode(command_hex) {
            self.write(0x000C, &bytes).await?;
        }
        Ok(())
    }

    pub async fn send_message(&self, message_str: &str) -> Result<(), Box<dyn std::error::Error>> {
        let commands = crate::message_dispatcher::MessageDispatcher::action_request(message_str);
        for command in commands {
            if let crate::iolib::actions::BleAction::Write(write_cmd) = command {
                self.write(write_cmd.handle, &write_cmd.data).await?;
            }
        }
        Ok(())
    }
}
