use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    extract::{Extension, State},
    http::HeaderMap,
};
use tower_http::request_id::RequestId;
use tracing::info;

use crate::{
    headers::{
        HEADER_ACCESS_TOKEN, HEADER_BATTERY_VOLTAGE, HEADER_FW_VERSION, HEADER_REFRESH_RATE,
        HEADER_RSSI,
    },
    models::DisplayResponse,
    state::AppState,
    utils::{get_header, request_id_to_string},
};

const DEFAULT_REFRESH_RATE: &str = "1800";

pub async fn display_handler(
    headers: HeaderMap,
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
) -> Json<DisplayResponse> {
    // Extract headers safely
    let access_token = get_header(&headers, &HEADER_ACCESS_TOKEN);
    let rssi = get_header(&headers, &HEADER_RSSI);
    let fw_version = get_header(&headers, &HEADER_FW_VERSION);
    let battery_voltage = get_header(&headers, &HEADER_BATTERY_VOLTAGE);
    let refresh_rate_raw = get_header(&headers, &HEADER_REFRESH_RATE);
    let refresh_rate = if refresh_rate_raw.is_empty() {
        DEFAULT_REFRESH_RATE.to_string()
    } else {
        refresh_rate_raw
    };

    // Generate filename based on current time
    let filename = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Lookup device by API key
    let device = sqlx::query!(
        "SELECT mac, friendly_id FROM devices WHERE api_key = ?",
        access_token
    )
    .fetch_optional(&*state.db)
    .await
    .unwrap_or(None);

    match device {
        Some(dev) => {
            info!(
                msg = "Processing display request",
                req_id = %request_id_to_string(&req_id),
                mac = ?dev.mac,
                %rssi, %fw_version, %battery_voltage, %refresh_rate, %filename
            );

            Json(DisplayResponse {
                status: 0,
                image_url: state.config.app.setup_logo_url.clone(),
                filename,
                update_firmware: false,
                firmware_url: None,
                refresh_rate,
                reset_firmware: false,
            })
        }
        None => {
            info!(
                msg = "Rejecting display request",
                req_id = %request_id_to_string(&req_id),
                access_token = %access_token,
                %rssi, %fw_version, %battery_voltage, %refresh_rate
            );

            Json(DisplayResponse {
                status: 500,
                image_url: state.config.app.setup_logo_url.clone(),
                filename,
                update_firmware: false,
                firmware_url: None,
                refresh_rate,
                reset_firmware: false,
            })
        }
    }
}
