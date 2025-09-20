use crate::repositories::device::DeviceRepo;

use axum::{Extension, Json, extract::Path, http::StatusCode};

pub async fn put_device_images_handler(
    Extension(device_repo): Extension<DeviceRepo>,
    Path(id): Path<String>,
    Json(images): Json<Vec<String>>,
) -> Result<Json<Vec<String>>, (StatusCode, &'static str)> {
    device_repo
        .update_images(&id, &images)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong"))?;

    Ok(Json(images))
}
