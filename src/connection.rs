use tokio::sync::mpsc;

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

pub struct Connection {
    #[cfg(feature = "ble")]
    adapter: Option<Adapter>,
    #[cfg(feature = "ble")]
    peripheral: Option<Peripheral>,
    notification_rx: Option<mpsc::UnboundedReceiver<Vec<u8>>>,
    _notification_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl Connection {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            #[cfg(feature = "ble")]
            adapter: None,
            #[cfg(feature = "ble")]
            peripheral: None,
            notification_rx: Some(rx),
            _notification_tx: tx,
        }
    }

    pub fn take_notification_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<Vec<u8>>> {
        self.notification_rx.take()
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
                    if name.starts_with("CASIO")
                        || name.contains("G-SHOCK")
                        || name.contains("ECB")
                        || name.contains("EQB")
                        || name.contains("GST")
                        || name.contains("GBD")
                        || name.contains("GBX")
                        || name.contains("GPR")
                        || name.contains("MRG")
                        || name.contains("GMA")
                    {
                        println!("Found Casio watch: {}", name);
                        if peripheral.connect().await.is_ok() {
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
                                let tx = self._notification_tx.clone();
                                tokio::spawn(async move {
                                    while let Some(notif) = stream.next().await {
                                        let _ = tx.send(notif.value);
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
                peripheral.write(characteristic, data, write_type).await?;
            }
        }
        Ok(())
    }

    #[cfg(not(feature = "ble"))]
    pub async fn write(&self, handle: u16, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        println!("Simulated write: handle=0x{:04X}, data={:02X?}", handle, data);
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
