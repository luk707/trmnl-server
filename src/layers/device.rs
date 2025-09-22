use axum::{Extension, middleware::AddExtension};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::Layer;

use crate::repositories::device::{DeviceRepo, SqliteDeviceRepo};

#[derive(Clone)]
pub struct DeviceRepoLayer(pub DeviceRepo);

impl DeviceRepoLayer {
    pub fn sqlite(pool: Arc<SqlitePool>) -> Self {
        Self(Arc::new(SqliteDeviceRepo::new(pool)))
    }
}

impl<S> Layer<S> for DeviceRepoLayer {
    type Service = AddExtension<S, DeviceRepo>;

    fn layer(&self, inner: S) -> Self::Service {
        Extension(self.0.clone()).layer(inner)
    }
}
