use crate::state::AppState;
use axum::{Json, extract::Path, extract::State};

pub async fn put_device_images_handler(
    Path(friendly_id): Path<String>,
    State(state): State<AppState>,
    Json(images): Json<Vec<String>>,
) -> Json<Vec<String>> {
    // Serialize the array to JSON
    let json_str = serde_json::to_string(&images).unwrap_or_else(|_| "[]".to_string());

    // Update the database
    let _ = sqlx::query!(
        "UPDATE devices SET images_json = ? WHERE friendly_id = ?",
        json_str,
        friendly_id
    )
    .execute(&*state.db)
    .await;

    // Return the new list to confirm
    Json(images)
}
