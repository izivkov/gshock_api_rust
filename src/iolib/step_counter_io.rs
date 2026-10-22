pub use crate::model::step_counter_data::{
    DailyHistoryEntry, HourlyActivity, PACKET_HEADER_MARKER, SENTINEL_BUCKET_VALUE,
    SENTINEL_DAILY_VALUE, StepCounterData, decode_bcd,
};

use std::time::Duration;

use tokio::time::timeout;

use crate::connection::{CharacteristicNotification, Connection};

pub const DRSP_CATEGORY_EXERCISE: u8 = 0x11;
pub const START_TRANSACTION_CMD: [u8; 5] = [0x00, DRSP_CATEGORY_EXERCISE, 0x00, 0x00, 0x00];
pub const END_TRANSACTION_CMD: [u8; 5] = [0x04, DRSP_CATEGORY_EXERCISE, 0x00, 0x00, 0x00];
pub const BLE_HANDLE_DRSP: u16 = 0x0011;
pub const BLE_HANDLE_CONVOY: u16 = 0x0014;
pub const FALLBACK_EXPECTED_LENGTH: usize = 400;
const MAX_EXPECTED_LENGTH: usize = 65_536;
const TRANSFER_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct StepCounterTransfer {
    expected_length: usize,
    payload: Vec<u8>,
}

impl Default for StepCounterTransfer {
    fn default() -> Self {
        Self {
            expected_length: FALLBACK_EXPECTED_LENGTH,
            payload: Vec::with_capacity(FALLBACK_EXPECTED_LENGTH),
        }
    }
}

impl StepCounterTransfer {
    pub fn expected_length(&self) -> usize {
        self.expected_length
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn handle_notification(&mut self, notification: &CharacteristicNotification) -> bool {
        match notification.handle {
            BLE_HANDLE_DRSP => self.handle_drsp(&notification.data),
            BLE_HANDLE_CONVOY => self.handle_fragment(&notification.data),
            _ => false,
        }
    }

    fn handle_drsp(&mut self, data: &[u8]) -> bool {
        if data.len() < 5 || data[1] != DRSP_CATEGORY_EXERCISE || data[0] != 0x00 {
            return false;
        }
        let announced =
            usize::from(data[2]) | (usize::from(data[3]) << 8) | (usize::from(data[4]) << 16);
        if announced > 0 && announced <= MAX_EXPECTED_LENGTH {
            self.expected_length = announced;
            self.payload
                .reserve(announced.saturating_sub(self.payload.len()));
        }
        self.is_complete()
    }

    fn handle_fragment(&mut self, data: &[u8]) -> bool {
        self.payload.extend_from_slice(data);
        self.is_complete()
    }

    pub fn is_complete(&self) -> bool {
        self.payload.len() >= self.expected_length
    }
}

pub async fn request(
    connection: &Connection,
    peek: bool,
) -> Result<StepCounterData, Box<dyn std::error::Error>> {
    let mut notifications = connection.subscribe_characteristic_notifications();
    let mut transfer = StepCounterTransfer::default();
    connection
        .write(BLE_HANDLE_DRSP, &START_TRANSACTION_CMD)
        .await?;

    let completed = timeout(TRANSFER_TIMEOUT, async {
        loop {
            match notifications.recv().await {
                Ok(notification) => {
                    if transfer.handle_notification(&notification) && transfer.is_complete() {
                        break true;
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break false,
            }
        }
    })
    .await
    .unwrap_or(false);

    if !peek && completed {
        let _ = connection
            .write(BLE_HANDLE_DRSP, &END_TRANSACTION_CMD)
            .await;
    }

    if completed {
        Ok(StepCounterData::parse(transfer.payload()).unwrap_or_else(StepCounterData::unavailable))
    } else {
        Ok(StepCounterData::unavailable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn reads_drsp_length_announcement_and_collects_fragments() {
        let mut transfer = StepCounterTransfer::default();
        assert_eq!(transfer.expected_length(), FALLBACK_EXPECTED_LENGTH);
        assert!(!transfer.handle_notification(&CharacteristicNotification {
            handle: BLE_HANDLE_DRSP,
            data: vec![0x00, DRSP_CATEGORY_EXERCISE, 0x08, 0x00, 0x00],
        }));
        assert_eq!(transfer.expected_length(), 8);
        assert!(!transfer.handle_notification(&CharacteristicNotification {
            handle: BLE_HANDLE_CONVOY,
            data: vec![0x26, 1, 2, 3],
        }));
        assert!(transfer.handle_notification(&CharacteristicNotification {
            handle: BLE_HANDLE_CONVOY,
            data: vec![4, 5, 6, 7],
        }));
        assert_eq!(transfer.payload(), &[0x26, 1, 2, 3, 4, 5, 6, 7]);
    }

    #[tokio::test]
    async fn request_assembles_fragments_into_step_data() {
        let connection = Arc::new(Connection::new());
        let sender = Arc::clone(&connection);
        let producer = tokio::spawn(async move {
            tokio::task::yield_now().await;
            let mut payload = vec![0u8; FALLBACK_EXPECTED_LENGTH];
            payload[..6].copy_from_slice(&[0x26, 0x01, 0x08, 0x17, 0x30, 0x00]);
            payload[374..378].copy_from_slice(&321u32.to_le_bytes());
            sender.send_characteristic_notification(
                BLE_HANDLE_DRSP,
                vec![0x00, DRSP_CATEGORY_EXERCISE, 0x90, 0x01, 0x00],
            );
            for fragment in payload.chunks(100) {
                sender.send_characteristic_notification(BLE_HANDLE_CONVOY, fragment.to_vec());
            }
        });

        let data = request(&connection, true).await.unwrap();
        producer.await.unwrap();
        assert_eq!(data.current_day_steps, Some(321));
        assert_eq!(data.year, Some(2026));
    }
}
