use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    body::Bytes,
    extract::State,
    http::HeaderMap,
    routing::{get, post},
};
use config::Config;
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::UtcOffset;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
struct DatabaseSettings {
    path: String,
}

#[derive(Debug, Deserialize, Clone)]
struct AppSettings {
    setup_logo_url: String,
}

#[derive(Debug, Deserialize, Clone)]
struct ServerConfig {
    database: DatabaseSettings,
    app: AppSettings,
}

#[derive(Clone)]
struct AppState {
    db: Arc<SqlitePool>,
    config: ServerConfig,
}

#[derive(Serialize)]
struct SetupResponse {
    status: u16,
    api_key: String,
    friendly_id: String,
    image_url: String,
    filename: String,
}

#[derive(Serialize)]
struct DisplayResponse {
    status: u16,
    image_url: String,
    filename: String,
    update_firmware: bool,
    firmware_url: Option<String>,
    refresh_rate: String,
    reset_firmware: bool,
}

#[derive(Serialize)]
struct DisplayError {
    status: u16,
    error: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use UTC timestamps
    let offset = UtcOffset::UTC;

    // Install tracing subscriber with JSON formatter
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .json()
                .with_timer(fmt::time::OffsetTime::new(
                    offset,
                    time::format_description::well_known::Rfc3339,
                ))
                .with_level(true) // "level": "info"
                .with_target(false) // don’t log module path
                .with_thread_ids(false)
                .with_thread_names(false),
        )
        .with(EnvFilter::from_default_env())
        .try_init()?;

    // Load config from `config.toml`
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?
        .try_deserialize::<ServerConfig>()?;

    info!(msg = "Loaded config", ?settings);

    // DB connection pool
    let pool = SqlitePool::connect(&settings.database.path).await?;

    info!(msg = "Database initialized", path = %settings.database.path);

    let state = AppState {
        db: Arc::new(pool),
        config: settings.clone(),
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/setup", get(setup_handler))
        .route("/api/display", get(display_handler))
        .route("/api/logs", post(logs_handler))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!(msg = "Server starting", addr = "0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn setup_handler(headers: HeaderMap, State(state): State<AppState>) -> Json<SetupResponse> {
    let mac = headers
        .get("ID")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    info!(msg = "Received setup request", mac = %mac);

    let api_key: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(22)
        .map(char::from)
        .collect();

    let existing = sqlx::query!(
        "SELECT mac, api_key, friendly_id FROM devices WHERE mac = ?",
        mac
    )
    .fetch_optional(&*state.db)
    .await
    .unwrap();

    let friendly_id: String;

    if let Some(device) = existing {
        // Keep same friendly_id, replace api_key
        friendly_id = device.friendly_id;

        sqlx::query!("UPDATE devices SET api_key = ? WHERE mac = ?", api_key, mac)
            .execute(&*state.db)
            .await
            .unwrap();
    } else {
        // New device
        friendly_id = Uuid::new_v4().simple().to_string()[..6].to_uppercase();

        sqlx::query!(
            "INSERT INTO devices (mac, api_key, friendly_id) VALUES (?, ?, ?)",
            mac,
            api_key,
            friendly_id,
        )
        .execute(&*state.db)
        .await
        .unwrap();
    }

    info!(msg = "Setup complete", mac = %mac, friendly_id = %friendly_id);

    Json(SetupResponse {
        status: 200,
        api_key,
        friendly_id,
        image_url: state.config.app.setup_logo_url.clone(),
        filename: "empty_state".to_string(),
    })
}

async fn display_handler(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    let mac = headers
        .get("ID")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let access_token = headers
        .get("Access-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let rssi = headers
        .get("RSSI")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let fw_version = headers
        .get("FW-Version")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let battery_voltage = headers
        .get("Battery-Voltage")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default()
        .to_string();

    let refresh_rate = headers
        .get("Refresh-Rate")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("1800")
        .to_string();

    // Lookup device by api_key
    let device = sqlx::query!(
        "SELECT mac, friendly_id FROM devices WHERE api_key = ?",
        access_token
    )
    .fetch_optional(&*state.db)
    .await
    .unwrap();

    match device {
        Some(dev) => {
            let filename = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string();

            info!(
                msg = "Display request success",
                mac = ?dev.mac,
                rssi = %rssi,
                fw_version = %fw_version,
                battery_voltage = %battery_voltage,
                refresh_rate = %refresh_rate,
                filename = %filename
            );

            Json(serde_json::json!({
                "status": 0,
                "image_url": state.config.app.setup_logo_url.clone(),
                "filename": filename,
                "update_firmware": false,
                "firmware_url": serde_json::Value::Null,
                "refresh_rate": "1800",
                "reset_firmware": false
            }))
        }
        None => {
            info!(
                msg = "Display request failed",
                mac = %mac,
            );

            Json(serde_json::json!({
                "status": 500,
                "error": "Device not found"
            }))
        }
    }
}

async fn logs_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Json<serde_json::Value> {
    // Convert raw body to string safely
    let body_str = String::from_utf8_lossy(&body);

    // Optionally extract some key headers
    let mac = headers
        .get("ID")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    let access_token = headers
        .get("Access-Token")
        .and_then(|h| h.to_str().ok())
        .unwrap_or_default();

    info!(
        msg = "Received /api/logs request",
        mac = %mac,
        access_token = %access_token,
        headers = ?headers,
        body = %body_str
    );

    // Always respond with 200 OK
    Json(serde_json::json!({
        "status": 200,
        "msg": "log received"
    }))
}
