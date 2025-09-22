use axum::{
    Router,
    routing::{get, post, put},
};

use crate::handlers::{
    display_handler, get_device_handler, get_device_images_handler, list_devices_handler,
    log_handler, put_device_images_handler, setup_handler,
};

pub struct App;

impl App {
    pub fn new() -> Self {
        Self
    }
    pub fn router(self) -> Router {
        Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/api/setup", get(setup_handler))
            .route("/api/display", get(display_handler))
            .route("/api/log", post(log_handler))
            .route("/api/devices", get(list_devices_handler))
            .route("/api/devices/{id}", get(get_device_handler))
            .route("/api/devices/{id}/images", get(get_device_images_handler))
            .route("/api/devices/{id}/images", put(put_device_images_handler))
    }
}
