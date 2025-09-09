use std::{
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router, ServiceExt,
    body::{Body, Bytes},
    extract::{Request, State},
    http::{HeaderMap, Response, StatusCode},
    routing::{get, post},
};
use config::Config;
use rand::{Rng, distr::Alphanumeric};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use time::UtcOffset;
use tower::Layer;
use tower_http::{
    normalize_path::NormalizePathLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, RequestId, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::{Span, info};
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
                .flatten_event(true)
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
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(&settings.database.path)
            .create_if_missing(true),
    )
    .await?;

    // Apply migrations
    apply_migrations(&pool).await?;

    info!(msg = "Database initialized", path = %settings.database.path);

    let state = AppState {
        db: Arc::new(pool),
        config: settings.clone(),
    };

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/setup", get(setup_handler))
        .route("/api/display", get(display_handler))
        .route("/api/log", post(log_handler))
        .with_state(state)
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .on_request(|req: &Request<Body>, _span: &Span| {
                    let id = req
                        .extensions()
                        .get::<RequestId>()
                        .and_then(|req_id| req_id.header_value().to_str().ok())
                        .unwrap_or_default()
                        .to_string();

                    info!(
                        msg = "Request initiated",
                        req_id = %id,
                        method = %req.method(),
                        uri = %req.uri()
                    )
                })
                .on_response(|res: &Response<Body>, latency: Duration, _span: &Span| {
                    let id = res
                        .extensions()
                        .get::<RequestId>()
                        .and_then(|req_id| req_id.header_value().to_str().ok())
                        .unwrap_or_default()
                        .to_string();

                    info!(
                        msg = "Request processed",
                        req_id = %id,
                        status = %res.status().as_u16(),
                        latency = ?latency
                    )
                }),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()));

    let app = NormalizePathLayer::trim_trailing_slash().layer(app);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!(msg = "Server starting", addr = "0.0.0.0:3000");

    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .unwrap();

    Ok(())
}

async fn apply_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    let before_count: i64 = match sqlx::query_scalar!("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
    {
        Ok(count) => count,
        Err(sqlx::Error::Database(_)) => 0,
        Err(e) => return Err(e.into()),
    };

    sqlx::migrate!("./migrations").run(pool).await?;

    let after_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;

    let limit = after_count - before_count;

    let new_migrations = sqlx::query!(
        "SELECT version, description FROM _sqlx_migrations ORDER BY version DESC LIMIT ?",
        limit
    )
    .fetch_all(pool)
    .await?;

    for m in new_migrations {
        info!(
            msg = "New migration applied",
            version = %m.version.map(|v| v.to_string()).unwrap_or_default(),
            description = %m.description,
        );
    }

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
) -> Result<Json<DisplayResponse>, (StatusCode, Json<DisplayError>)> {
    let mac_header = headers
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

    // No API key → return default response
    if access_token.is_empty() {
        let filename = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        info!(
            msg = "Display request with no API key",
            mac = %mac_header,
            rssi = %rssi,
            fw_version = %fw_version,
            battery_voltage = %battery_voltage,
            refresh_rate = %refresh_rate,
            filename = %filename
        );

        return Ok(Json(DisplayResponse {
            status: 0,
            image_url: state.config.app.setup_logo_url.clone(),
            filename,
            update_firmware: false,
            firmware_url: None,
            refresh_rate: "1800".to_string(),
            reset_firmware: false,
        }));
    }

    // Lookup device by API key (ignore mac)
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

            Ok(Json(DisplayResponse {
                status: 0,
                image_url: state.config.app.setup_logo_url.clone(),
                filename,
                update_firmware: false,
                firmware_url: None,
                refresh_rate: "1800".to_string(),
                reset_firmware: false,
            }))
        }
        None => {
            info!(
                msg = "Display request failed",
                access_token = %access_token,
                rssi = %rssi,
                fw_version = %fw_version,
                battery_voltage = %battery_voltage,
                refresh_rate = %refresh_rate,
            );

            Err((
                StatusCode::NOT_FOUND,
                Json(DisplayError {
                    status: 404,
                    error: String::from("Device not found"),
                }),
            ))
        }
    }
}

async fn log_handler(
    State(_state): State<AppState>,
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
