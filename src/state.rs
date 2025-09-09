use std::collections::HashMap;
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::Mutex;

use crate::config::ServerConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<SqlitePool>,
    pub config: ServerConfig,
    pub image_counters: Arc<Mutex<HashMap<String, usize>>>,
}
