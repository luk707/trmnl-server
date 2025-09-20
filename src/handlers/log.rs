use axum::{Json, body::Bytes};

pub async fn log_handler(body: Bytes) -> Json<serde_json::Value> {
    // Just grab body for future DB insert
    let _body_str = String::from_utf8_lossy(&body);

    Json(serde_json::json!({
        "status": 200,
        "msg": "log received"
    }))
}
