use crate::state::AppState;
use axum::{Json, extract::Path, extract::State};

fn parse_images(json: &str) -> Vec<String> {
    serde_json::from_str(json).unwrap_or_default()
}

pub async fn get_device_images_handler(
    Path(friendly_id): Path<String>,
    State(state): State<AppState>,
) -> Json<Vec<String>> {
    let record = sqlx::query!(
        "SELECT images_json FROM devices WHERE friendly_id = ?",
        friendly_id
    )
    .fetch_optional(&*state.db)
    .await
    .unwrap();

    let images = record
        .map(|r| parse_images(&r.images_json))
        .unwrap_or_default();

    Json(images)
}
