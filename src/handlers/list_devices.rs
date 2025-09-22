use crate::{models::DeviceInfo, repositories::device::DeviceRepo};
use axum::Extension;
use axum::Json;
use axum::http::StatusCode;
use tracing::instrument;

#[instrument(name = "handlers.list_devices", skip(device_repo))]
pub async fn list_devices_handler(
    Extension(device_repo): Extension<DeviceRepo>,
) -> Result<Json<Vec<DeviceInfo>>, (StatusCode, &'static str)> {
    let devices = device_repo
        .list()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"))?;

    Ok(Json(
        devices
            .iter()
            .map(|device| DeviceInfo {
                id: device.id.clone(),
                mac: device.mac.clone(),
                rssi: device.rssi,
                battery_voltage: device.battery_voltage,
                fw_version: device.fw_version.clone(),
                refresh_rate: device.refresh_rate,
            })
            .collect(),
    ))
}
