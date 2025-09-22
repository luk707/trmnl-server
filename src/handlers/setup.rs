use axum::{
    Json,
    extract::Extension,
    http::{HeaderMap, StatusCode},
};
use rand::{Rng, distr::Alphanumeric};
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
    config::AppSettings, headers::HEADER_MAC, models::SetupResponse,
    repositories::device::DeviceRepo, utils::get_optional_header,
};

#[instrument(name = "handlers.setup", skip(headers, device_repo, settings))]
pub async fn setup_handler(
    headers: HeaderMap,
    Extension(device_repo): Extension<DeviceRepo>,
    Extension(settings): Extension<AppSettings>,
) -> Result<Json<SetupResponse>, (StatusCode, &'static str)> {
    let mac = get_optional_header(&headers, &HEADER_MAC);

    match mac {
        Some(mac)
            if device_repo
                .exists_by_mac(mac)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"))? =>
        {
            info!(msg = "Device setup attempted for existing device", ?mac);

            return Ok(Json(SetupResponse {
                status: 404,
                api_key: None,
                friendly_id: None,
                image_url: None,
                filename: None,
            }));
        }
        _ => {
            let api_key: String = rand::rng()
                .sample_iter(&Alphanumeric)
                .take(22)
                .map(char::from)
                .collect();

            let id = Uuid::new_v4().simple().to_string()[..6].to_uppercase();

            // Insert into DB
            device_repo
                .create(&id, mac, &api_key)
                .await
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"))?;

            info!(
                msg = "Device successfully registered",
                ?mac,
                %id
            );

            Ok(Json(SetupResponse {
                status: 200,
                api_key: Some(api_key),
                friendly_id: Some(id),
                image_url: Some(settings.setup_logo_url.clone()),
                filename: Some("empty_state".to_string()),
            }))
        }
    }
}
