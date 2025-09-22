use std::{iter::once, sync::Arc};

use axum::{Extension, ServiceExt, extract::Request, http::HeaderName};
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    trace::{BatchConfig, BatchSpanProcessor, TracerProviderBuilder},
};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    sensitive_headers::SetSensitiveRequestHeadersLayer,
    trace::TraceLayer,
};
use tracing::{Span, info};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use trmnl_server::{
    app::App,
    config::{LogFormat, ServerConfig},
    db::{apply_migrations, connect},
    layers::device::DeviceRepoLayer,
    utils::get_request_id,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let settings = ServerConfig::load()?;
    info!(
        msg = "Loaded configuration",
        database_path = settings.database.path,
        setup_logo_url = settings.app.setup_logo_url
    );

    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint("http://localhost:4318/v1/traces")
        .build()
        .expect("Failed to create span exporter");

    let provider = TracerProviderBuilder::default()
        .with_resource(
            Resource::builder()
                .with_service_name("trmnl-server")
                .build(),
        )
        .with_span_processor(BatchSpanProcessor::new(exporter, BatchConfig::default()))
        .build();

    let tracer = provider.tracer("trmnl-server");

    let otel_layer = OpenTelemetryLayer::new(tracer); // Now the trait bound is satisfied

    tracing_subscriber::registry()
        .with(match settings.logging.format {
            LogFormat::Json => fmt::layer().json().flatten_event(true).boxed(),
            LogFormat::Pretty => fmt::layer().boxed(),
        })
        .with(EnvFilter::from_default_env())
        .with(otel_layer)
        .try_init()?;

    let pool = Arc::new(connect(&settings.database.path).await?);
    apply_migrations(&pool).await?;
    info!(msg = "Initialized database", path = %settings.database.path);

    let app = App::new()
        .router()
        .layer(Extension(ServerConfig::load()?))
        .layer(DeviceRepoLayer::sqlite(pool.clone()))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|req: &Request<_>| {
                    tracing::info_span!(
                        "http.request",
                        method = %req.method(),
                        uri = %req.uri(),
                        req_id = %get_request_id(req.extensions())
                    )
                })
                .on_response(
                    |res: &axum::response::Response<_>,
                     latency: std::time::Duration,
                     span: &Span| {
                        span.in_scope(|| {
                            if res.status().is_server_error() {
                                tracing::error!(
                                    status = %res.status().as_u16(),
                                    latency_ms = %latency.as_millis(),
                                )
                            } else {
                                tracing::info!(
                                    status = %res.status().as_u16(),
                                    latency_ms = %latency.as_millis(),
                                )
                            }
                        });
                    },
                ),
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
