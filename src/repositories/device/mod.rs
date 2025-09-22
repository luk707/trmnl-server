use async_trait::async_trait;
use mockall::automock;

use crate::models::Device;

pub mod sqlite;
pub use sqlite::SqliteDeviceRepo;

#[async_trait]
#[automock]
pub trait DeviceRepository: Send + Sync {
    /// Create a new device
    async fn create(&self, id: &str, mac: Option<&str>, api_key: &str) -> anyhow::Result<()>;

    /// Check if a device exists by its MAC address
    async fn exists_by_mac(&self, mac: &str) -> anyhow::Result<bool>;

    /// Get a device by its API key
    async fn get_by_api_key(&self, api_key: &str) -> anyhow::Result<Option<Device>>;

    /// Get a device by its ID
    async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<Device>>;

    /// List all devices
    async fn list(&self) -> anyhow::Result<Vec<Device>>;

    /// Update device images
    async fn update_images(&self, id: &str, images: &[String]) -> anyhow::Result<()>;

    /// Update device status
    async fn update_status(
        &self,
        id: &str,
        rssi: Option<i32>,
        battery_voltage: Option<f32>,
        fw_version: Option<&str>,
        refresh_rate: Option<i32>,
    ) -> anyhow::Result<()>;

    // /// Get a device by its MAC address
    // async fn get_by_mac(&self, mac: &str) -> anyhow::Result<Option<Device>>;

    // /// Update device fields (mac/api_key)
    // async fn update(&self, ...) -> anyhow::Result<()>;

    // /// Delete a device by its ID
    // async fn delete(&self, id: &str) -> anyhow::Result<()>;

    // /// Check if a device exists by its ID (works for physical devices)
    // async fn exists_by_id(&self, id: &str) -> anyhow::Result<bool>;

    // /// Count all devices (useful for stats)
    // async fn count(&self) -> anyhow::Result<u64>;
}

pub type DeviceRepo = std::sync::Arc<dyn DeviceRepository + Send + Sync>;
