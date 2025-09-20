use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use tower_http::request_id::RequestId;
use tracing::info;

use crate::{models::DeviceInfo, state::AppState, utils::request_id_to_string};

pub async fn get_device_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
) -> Result<Json<DeviceInfo>, (StatusCode, String)> {
    // Lookup device by id
    let device = sqlx::query!(
        "SELECT mac, id, rssi, battery_voltage, fw_version, refresh_rate
         FROM devices
         WHERE id = ?",
        id
    )
    .fetch_optional(&*state.db)
    .await
    .unwrap_or(None);

    match device {
        Some(d) => {
            info!(
                msg = "Fetched device",
                req_id = %request_id_to_string(&req_id),
                id = %d.id,
                mac = ?d.mac,
            );

            Ok(Json(DeviceInfo {
                id: d.id,
                mac: d.mac,
                rssi: d.rssi,
                battery_voltage: d.battery_voltage,
                fw_version: d.fw_version,
                refresh_rate: d.refresh_rate,
            }))
        }
        None => {
            info!(
                msg = "Device not found",
                req_id = %request_id_to_string(&req_id),
                id = %id
            );

            Err((StatusCode::NOT_FOUND, format!("Device {} not found", id)))
        }
    }
}
