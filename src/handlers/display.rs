use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    Json,
    extract::Extension,
    http::{HeaderMap, StatusCode},
};
use tower_http::request_id::RequestId;
use tracing::info;

use crate::{
    config::ServerConfig,
    headers::{
        HEADER_ACCESS_TOKEN, HEADER_BATTERY_VOLTAGE, HEADER_FW_VERSION, HEADER_REFRESH_RATE,
        HEADER_RSSI,
    },
    models::DisplayResponse,
    repositories::device::DeviceRepo,
    utils::{get_header, request_id_to_string},
};

const DEFAULT_REFRESH_RATE: &str = "1800";

pub async fn display_handler(
    headers: HeaderMap,
    Extension(req_id): Extension<RequestId>,
    Extension(device_repo): Extension<DeviceRepo>,
    Extension(config): Extension<ServerConfig>,
) -> Result<Json<DisplayResponse>, (StatusCode, &'static str)> {
    let access_token = get_header(&headers, &HEADER_ACCESS_TOKEN);
    let rssi = get_header(&headers, &HEADER_RSSI);
    let fw_version = get_header(&headers, &HEADER_FW_VERSION);
    let battery_voltage = get_header(&headers, &HEADER_BATTERY_VOLTAGE);
    let refresh_rate_raw = get_header(&headers, &HEADER_REFRESH_RATE);
    let refresh_rate = if refresh_rate_raw.is_empty() {
        DEFAULT_REFRESH_RATE
    } else {
        refresh_rate_raw
    };

    // Generate filename based on current time
    let filename = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    if let Some(device) = device_repo
        .get_by_api_key(access_token)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"))?
    {
        device_repo
            .update_status(
                &device.id,
                rssi.parse().ok(),
                battery_voltage.parse().ok(),
                if fw_version.is_empty() {
                    None
                } else {
                    Some(&fw_version)
                },
                refresh_rate.parse().ok(),
            )
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"))?;

        info!(
            msg = "Processing display request",
            req_id = %request_id_to_string(&req_id),
            id = %device.id,
            mac = ?device.mac,
            %rssi,
            %fw_version,
            %battery_voltage,
            %refresh_rate,
            %filename
        );

        return Ok(Json(DisplayResponse {
            status: 0,
            image_url: config.app.setup_logo_url.clone(),
            filename,
            update_firmware: false,
            firmware_url: None,
            refresh_rate: refresh_rate.to_string(),
            reset_firmware: false,
        }));
    }

    info!(
        msg = "Rejecting display request",
        req_id = %request_id_to_string(&req_id),
        access_token = %access_token,
        %rssi,
        %fw_version,
        %battery_voltage,
        %refresh_rate
    );

    Ok(Json(DisplayResponse {
        status: 500,
        image_url: config.app.setup_logo_url.clone(),
        filename,
        update_firmware: false,
        firmware_url: None,
        refresh_rate: refresh_rate.to_string(),
        reset_firmware: false,
    }))
}
