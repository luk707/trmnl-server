use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    Extension,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use mockall::predicate;
use serde_json::Value;
use tower::ServiceExt;
use trmnl_server::{
    app::App, config::AppSettings, headers::HEADER_MAC, layers::device::DeviceRepoLayer,
    repositories::device::MockDeviceRepository,
};

#[tokio::test]
async fn success_already_exists() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_exists_by_mac()
        .with(predicate::eq("AA:BB:CC:DD:EE:FF".to_string()))
        .times(1)
        .returning(|_| Box::pin(async { Ok(true) }));

    let settings = AppSettings {
        setup_logo_url: "http://example.com/logo.png".to_string(),
    };

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(settings))
        .oneshot(
            Request::builder()
                .uri("/api/setup")
                .header(HEADER_MAC, "AA:BB:CC:DD:EE:FF")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], 404);
    assert!(json["api_key"].is_null());
}

#[tokio::test]
async fn success_created_physical() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_exists_by_mac()
        .with(predicate::eq("AA:BB:CC:DD:EE:FF".to_string()))
        .times(1)
        .returning(|_| Box::pin(async { Ok(false) }));

    mock_repo
        .expect_create()
        .times(1)
        .returning(|_id, _mac, _api_key| Box::pin(async { Ok(()) }));

    let settings = AppSettings {
        setup_logo_url: "http://example.com/logo.png".to_string(),
    };

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(settings))
        .oneshot(
            Request::builder()
                .uri("/api/setup")
                .header(HEADER_MAC, "AA:BB:CC:DD:EE:FF")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], 200);
    assert!(json["api_key"].is_string());
    assert!(json["friendly_id"].is_string());
    assert_eq!(json["image_url"], "http://example.com/logo.png");
    assert_eq!(json["filename"], "empty_state");
}

#[tokio::test]
async fn success_created_virtual() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo.expect_exists_by_mac().times(0);

    mock_repo
        .expect_create()
        .times(1)
        .returning(|_id, _mac, _api_key| Box::pin(async { Ok(()) }));

    let settings = AppSettings {
        setup_logo_url: "http://example.com/logo.png".to_string(),
    };

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(settings))
        .oneshot(
            Request::builder()
                .uri("/api/setup")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], 200);
    assert!(json["api_key"].is_string());
    assert!(json["friendly_id"].is_string());
    assert_eq!(json["image_url"], "http://example.com/logo.png");
    assert_eq!(json["filename"], "empty_state");
}

#[tokio::test]
async fn error_exists_by_mac() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_exists_by_mac()
        .with(predicate::eq("AA:BB:CC:DD:EE:FF".to_string()))
        .times(1)
        .returning(|_| Box::pin(async { Err(anyhow!("DB Error")) }));

    let settings = AppSettings {
        setup_logo_url: "http://example.com/logo.png".to_string(),
    };

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(settings))
        .oneshot(
            Request::builder()
                .uri("/api/setup")
                .header(HEADER_MAC, "AA:BB:CC:DD:EE:FF")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn error_create() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_exists_by_mac()
        .with(predicate::eq("AA:BB:CC:DD:EE:FF".to_string()))
        .times(1)
        .returning(|_| Box::pin(async { Ok(false) }));

    mock_repo
        .expect_create()
        .times(1)
        .returning(|_id, _mac, _api_key| Box::pin(async { Err(anyhow!("DB Error")) }));

    let settings = AppSettings {
        setup_logo_url: "http://example.com/logo.png".to_string(),
    };

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(settings))
        .oneshot(
            Request::builder()
                .uri("/api/setup")
                .header(HEADER_MAC, "AA:BB:CC:DD:EE:FF")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
