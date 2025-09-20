use axum::{
    Json,
    extract::{Extension, Path},
    http::StatusCode,
};

use crate::{models::DeviceInfo, repositories::device::DeviceRepo};

pub async fn get_device_handler(
    Path(id): Path<String>,
    Extension(device_repo): Extension<DeviceRepo>,
) -> Result<Json<DeviceInfo>, (StatusCode, &'static str)> {
    match device_repo
        .get_by_id(&id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"))?
    {
        Some(device) => Ok(Json(DeviceInfo {
            id: device.id,
            mac: device.mac,
            rssi: device.rssi,
            battery_voltage: device.battery_voltage,
            fw_version: device.fw_version,
            refresh_rate: device.refresh_rate,
        })),
        _ => Err((StatusCode::NOT_FOUND, "Device not found")),
    }
}
