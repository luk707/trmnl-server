use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use mockall::predicate;
use tower::ServiceExt;
use trmnl_server::{
    app::App, layers::device::DeviceRepoLayer, models::Device,
    repositories::device::MockDeviceRepository,
};

#[tokio::test]
async fn success_found() {
    let device = Device {
        id: "dev123".to_string(),
        mac: Some("AA:BB:CC:DD:EE:FF".to_string()),
        _api_key: "abc123".to_string(),
        rssi: Some(-70),
        battery_voltage: Some(3.7),
        fw_version: Some("1.0.0".to_string()),
        refresh_rate: Some(60),
        images: vec!["image_one.jpg".to_string(), "image_two.jpg".to_string()],
    };

    let mut mock_device_repo = MockDeviceRepository::new();

    mock_device_repo
        .expect_get_by_id()
        .with(predicate::eq("dev123"))
        .times(1)
        .returning(move |_id| {
            let device = device.clone();
            Box::pin(async move { Ok(Some(device)) })
        });

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_device_repo)))
        .oneshot(
            Request::builder()
                .uri("/api/devices/dev123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let json: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap()).unwrap();

    assert_eq!(json["id"], "dev123");
    assert_eq!(json["mac"], "AA:BB:CC:DD:EE:FF");
}

#[tokio::test]
async fn success_not_found() {
    let mut mock_device_repo = MockDeviceRepository::new();

    mock_device_repo
        .expect_get_by_id()
        .with(predicate::eq("dev123"))
        .times(1)
        .returning(move |_id| Box::pin(async move { Ok(None) }));

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_device_repo)))
        .oneshot(
            Request::builder()
                .uri("/api/devices/dev123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn error() {
    let mut mock_device_repo = MockDeviceRepository::new();

    mock_device_repo
        .expect_get_by_id()
        .with(predicate::eq("dev123"))
        .times(1)
        .returning(move |_id| Box::pin(async move { Err(anyhow!("DB Error")) }));

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_device_repo)))
        .oneshot(
            Request::builder()
                .uri("/api/devices/dev123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
