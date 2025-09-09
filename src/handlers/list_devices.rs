use crate::models::DeviceInfo;
use crate::state::AppState;
use axum::{Json, extract::State};

pub async fn list_devices_handler(State(state): State<AppState>) -> Json<Vec<DeviceInfo>> {
    let devices = sqlx::query!(
        r#"
        SELECT friendly_id, mac, rssi, battery_voltage, fw_version, refresh_rate
        FROM devices
        ORDER BY friendly_id
        "#
    )
    .fetch_all(&*state.db)
    .await
    .unwrap_or_default();

    let result = devices
        .into_iter()
        .map(|d| DeviceInfo {
            id: d.friendly_id,
            mac: d.mac,
            rssi: d.rssi,
            battery_voltage: d.battery_voltage,
            fw_version: d.fw_version,
            refresh_rate: d.refresh_rate,
        })
        .collect();

    Json(result)
}
