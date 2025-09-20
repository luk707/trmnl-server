use axum::{
    Json,
    extract::{Extension, State},
    http::HeaderMap,
};
use rand::{Rng, distr::Alphanumeric};
use tower_http::request_id::RequestId;
use tracing::info;
use uuid::Uuid;

use crate::{
    headers::HEADER_MAC,
    models::SetupResponse,
    state::AppState,
    utils::{get_header, request_id_to_string},
};

pub async fn setup_handler(
    headers: HeaderMap,
    State(state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
) -> Json<SetupResponse> {
    let mac = get_header(&headers, &HEADER_MAC);

    // Check if device already exists
    let existing = sqlx::query!("SELECT mac FROM devices WHERE mac = ?", mac)
        .fetch_optional(&*state.db)
        .await
        .unwrap_or(None); // optional: handle DB errors gracefully

    if existing.is_some() {
        // Device already registered → respond with null fields
        info!(
            msg = "Device setup attempted for existing device",
            req_id = %request_id_to_string(&req_id),
            %mac
        );

        return Json(SetupResponse {
            status: 404,
            api_key: None,
            friendly_id: None,
            image_url: None,
            filename: None,
        });
    }

    // New device → generate API key and friendly ID
    let api_key: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(22)
        .map(char::from)
        .collect();

    let id = Uuid::new_v4().simple().to_string()[..6].to_uppercase();

    // Insert into DB
    let _ = sqlx::query!(
        "INSERT INTO devices (mac, api_key, id) VALUES (?, ?, ?)",
        mac,
        api_key,
        id
    )
    .execute(&*state.db)
    .await;

    info!(
        msg = "Device successfully registered",
        req_id = %request_id_to_string(&req_id),
        %mac,
        %id
    );

    Json(SetupResponse {
        status: 200,
        api_key: Some(api_key),
        friendly_id: Some(id),
        image_url: Some(state.config.app.setup_logo_url.clone()),
        filename: Some("empty_state".to_string()),
    })
}
