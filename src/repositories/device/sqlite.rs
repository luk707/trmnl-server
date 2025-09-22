use std::sync::Arc;

use async_trait::async_trait;
use sqlx::SqlitePool;
use tracing::instrument;

use crate::models::Device;

use super::DeviceRepository;

pub struct SqliteDeviceRepo(Arc<SqlitePool>);

impl SqliteDeviceRepo {
    pub fn new(pool: Arc<SqlitePool>) -> Self {
        Self(pool)
    }
}

#[async_trait]
impl DeviceRepository for SqliteDeviceRepo {
    #[instrument(name = "sqlite_device_repo.create", skip(self), fields(id))]
    async fn create(&self, id: &str, mac: Option<&str>, api_key: &str) -> anyhow::Result<()> {
        sqlx::query!(
            "INSERT INTO devices (mac, api_key, id) VALUES (?, ?, ?)",
            mac,
            api_key,
            id
        )
        .execute(&*self.0)
        .await?;

        Ok(())
    }

    #[instrument(name = "sqlite_device_repo.exists_by_mac", skip(self), fields(mac))]
    async fn exists_by_mac(&self, mac: &str) -> anyhow::Result<bool> {
        // COUNT(*) always returns one row, even if no devices match
        let record = sqlx::query!("SELECT COUNT(*) as count FROM devices WHERE mac = ?", mac)
            .fetch_one(&*self.0) // fetch_one instead of fetch_optional
            .await?;

        // unwrap_or(0) in case count is NULL (shouldn't happen, but safe)
        Ok(record.count > 0)
    }

    #[instrument(name = "sqlite_device_repo.get_by_api_key", skip(self))]
    async fn get_by_api_key(&self, api_key: &str) -> anyhow::Result<Option<Device>> {
        let device = sqlx::query!(
            r#"
            SELECT
                id,
                mac,
                api_key,
                rssi,
                battery_voltage,
                fw_version,
                refresh_rate,
                images_json
            FROM devices
            WHERE api_key = ?
            "#,
            api_key
        )
        .fetch_optional(&*self.0)
        .await?
        .map(|record| Device {
            id: record.id,
            mac: record.mac,
            _api_key: record.api_key,
            rssi: record.rssi,
            battery_voltage: record.battery_voltage,
            fw_version: record.fw_version,
            refresh_rate: record.refresh_rate,
            images: serde_json::from_str::<Vec<String>>(&record.images_json).unwrap_or_default(),
        });

        Ok(device)
    }

    #[instrument(name = "sqlite_device_repo.get_by_id", skip(self), fields(id))]
    async fn get_by_id(&self, id: &str) -> anyhow::Result<Option<Device>> {
        let device = sqlx::query!(
            r#"
            SELECT
                id,
                mac,
                api_key,
                rssi,
                battery_voltage,
                fw_version,
                refresh_rate,
                images_json
            FROM devices
            WHERE id = ?
            "#,
            id
        )
        .fetch_optional(&*self.0)
        .await?
        .map(|record| Device {
            id: record.id,
            mac: record.mac,
            _api_key: record.api_key,
            rssi: record.rssi,
            battery_voltage: record.battery_voltage,
            fw_version: record.fw_version,
            refresh_rate: record.refresh_rate,
            images: serde_json::from_str::<Vec<String>>(&record.images_json).unwrap_or_default(),
        });

        Ok(device)
    }

    #[instrument(name = "sqlite_device_repo.list", skip(self))]
    async fn list(&self) -> anyhow::Result<Vec<Device>> {
        Ok(sqlx::query!(
            r#"
                SELECT
                    id,
                    mac,
                    api_key,
                    rssi,
                    battery_voltage,
                    fw_version,
                    refresh_rate,
                    images_json
                FROM devices
                ORDER BY id
                "#
        )
        .fetch_all(&*self.0)
        .await?
        .iter()
        .map(|record| Device {
            id: record.id.clone(),
            mac: record.mac.clone(),
            _api_key: record.api_key.clone(),
            rssi: record.rssi,
            battery_voltage: record.battery_voltage,
            fw_version: record.fw_version.clone(),
            refresh_rate: record.refresh_rate,
            images: serde_json::from_str::<Vec<String>>(&record.images_json).unwrap_or_default(),
        })
        .collect())
    }

    #[instrument(name = "sqlite_device_repo.update_images", skip(self), fields(id))]
    async fn update_images(&self, id: &str, images: &[String]) -> anyhow::Result<()> {
        let json_str = serde_json::to_string(&images).unwrap_or_else(|_| "[]".to_string());

        sqlx::query!(
            "UPDATE devices SET images_json = ? WHERE id = ?",
            json_str,
            id
        )
        .execute(&*self.0)
        .await?;

        Ok(())
    }

    #[instrument(name = "sqlite_device_repo.update_status", skip(self), fields(id))]
    async fn update_status(
        &self,
        id: &str,
        rssi: Option<i32>,
        battery_voltage: Option<f32>,
        fw_version: Option<&str>,
        refresh_rate: Option<i32>,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE devices
            SET
                rssi = COALESCE(?, rssi),
                battery_voltage = COALESCE(?, battery_voltage),
                fw_version = COALESCE(?, fw_version),
                refresh_rate = COALESCE(?, refresh_rate)
            WHERE id = ?
            "#,
            rssi,
            battery_voltage,
            fw_version,
            refresh_rate,
            id
        )
        .execute(&*self.0)
        .await?;

        Ok(())
    }
}
