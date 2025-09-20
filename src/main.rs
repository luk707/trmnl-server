use std::{iter::once, sync::Arc, time::Duration};

use axum::{
    Extension, Router, ServiceExt,
    body::Body,
    extract::Request,
    http::{HeaderName, Response},
    routing::{get, post, put},
};
use time::UtcOffset;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::TraceLayer,
};
use tracing::{Span, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod handlers;
mod headers;
mod layers;
mod models;
mod repositories;
mod utils;

use crate::{
    config::ServerConfig,
    db::{apply_migrations, connect},
    handlers::{
        display_handler, get_device_handler, get_device_images_handler, list_devices_handler,
        log_handler, put_device_images_handler, setup_handler,
    },
    layers::device::DeviceRepoLayer,
    utils::get_request_id,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Use UTC timestamps
    let offset = UtcOffset::UTC;

    // Initialize tracing
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

    // Load configuration
    let settings = ServerConfig::load()?;
    info!(
        msg = "Loaded configuration",
        database_path = settings.database.path,
        setup_logo_url = settings.app.setup_logo_url
    );

    // Connect to DB
    let pool = Arc::new(connect(&settings.database.path).await?);
    apply_migrations(&pool).await?;
    info!(msg = "Initialized database", path = %settings.database.path);

    // Build routes
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/setup", get(setup_handler))
        .route("/api/display", get(display_handler))
        .route("/api/log", post(log_handler))
        .route("/api/devices", get(list_devices_handler))
        .route("/api/devices/{id}", get(get_device_handler))
        .route("/api/devices/{id}/images", get(get_device_images_handler))
        .route("/api/devices/{id}/images", put(put_device_images_handler))
        .layer(Extension(ServerConfig::load()?))
        .layer(DeviceRepoLayer::sqlite(pool.clone()))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .on_request(|req: &Request<Body>, _span: &Span| {
                    let headers = req
                        .headers()
                        .iter()
                        .filter(|(k, _)| k.as_str().to_ascii_lowercase() != "x-request-id")
                        .map(|(k, v)| {
                            let val = if v.is_sensitive() {
                                "******"
                            } else {
                                v.to_str().unwrap_or("<non-utf8>")
                            };
                            format!("{}: {}", k.as_str(), val)
                        })
                        .collect::<Vec<_>>()
                        .join("; ");

                    info!(
                        msg = "Request initiated",
                        req_id = %get_request_id(req.extensions()),
                        method = %req.method(),
                        uri = %req.uri(),
                        headers = %headers
                    )
                })
                .on_response(|res: &Response<Body>, latency: Duration, _span: &Span| {
                    info!(
                        msg = "Request processed",
                        req_id = %get_request_id(res.extensions()),
                        status = %res.status().as_u16(),
                        latency = ?latency
                    )
                }),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()))
        .layer(SetSensitiveRequestHeadersLayer::new(once(
            HeaderName::from_static("access-token"),
        )));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!(msg = "Starting server", addr = "0.0.0.0:3000");

    axum::serve(listener, ServiceExt::<Request>::into_make_service(app))
        .await
        .unwrap();

    Ok(())
}
