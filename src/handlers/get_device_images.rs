use axum::{Extension, Json, extract::Path, http::StatusCode};
use tracing::instrument;

use crate::repositories::device::DeviceRepo;

#[instrument(name = "handlers.get_device_images", skip(device_repo, id), fields(device_id = %id))]
pub async fn get_device_images_handler(
    Path(id): Path<String>,
    Extension(device_repo): Extension<DeviceRepo>,
) -> Result<Json<Vec<String>>, (StatusCode, &'static str)> {
    match device_repo
        .get_by_id(&id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"))?
    {
        Some(device) => Ok(Json(device.images)),
        _ => Err((StatusCode::NOT_FOUND, "Device not found")),
    }
}
