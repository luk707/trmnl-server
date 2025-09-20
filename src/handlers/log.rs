use axum::{
    Json,
    body::Bytes,
    extract::{Extension, State},
    http::HeaderMap,
};
use tower_http::request_id::RequestId;
use tracing::info;

use crate::{
    headers::{HEADER_ACCESS_TOKEN, HEADER_MAC},
    state::AppState,
    utils::{get_header, request_id_to_string},
};

pub async fn log_handler(
    State(_state): State<AppState>,
    Extension(req_id): Extension<RequestId>,
    headers: HeaderMap,
    body: Bytes,
) -> Json<serde_json::Value> {
    let mac = get_header(&headers, &HEADER_MAC);
    let access_token = get_header(&headers, &HEADER_ACCESS_TOKEN);

    // Just grab body for future DB insert
    let _body_str = String::from_utf8_lossy(&body);

    info!(
        msg = "Accepted logs request",
        req_id = %request_id_to_string(&req_id),
        %mac,
        %access_token
    );

    Json(serde_json::json!({
        "status": 200,
        "msg": "log received"
    }))
}
