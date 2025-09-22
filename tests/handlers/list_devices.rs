use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use trmnl_server::{
    app::App,
    layers::device::DeviceRepoLayer,
    models::{Device, DeviceInfo},
    repositories::device::MockDeviceRepository,
};

#[tokio::test]
async fn success() {
    let devices = vec![
        Device {
            id: "dev123".to_string(),
            mac: Some("AA:BB:CC:DD:EE:FF".to_string()),
            _api_key: "abc123".to_string(),
            rssi: Some(-70),
            battery_voltage: Some(3.7),
            fw_version: Some("1.0.0".to_string()),
            refresh_rate: Some(60),
            images: vec![],
        },
        Device {
            id: "dev456".to_string(),
            mac: None,
            _api_key: "def456".to_string(),
            rssi: Some(-60),
            battery_voltage: Some(3.8),
            fw_version: Some("1.1.0".to_string()),
            refresh_rate: Some(120),
            images: vec![],
        },
    ];

    let mut mock_repo = MockDeviceRepository::new();
    mock_repo.expect_list().times(1).returning(move || {
        let devices = devices.clone();
        Box::pin(async move { Ok(devices) })
    });

    let app = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/devices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Vec<DeviceInfo> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json.len(), 2);
    assert_eq!(json[0].id, "dev123");
    assert_eq!(json[0].mac.as_deref(), Some("AA:BB:CC:DD:EE:FF"));
    assert_eq!(json[1].id, "dev456");
    assert_eq!(json[1].mac, None);
}

#[tokio::test]
async fn error() {
    let mut mock_repo = MockDeviceRepository::new();
    mock_repo
        .expect_list()
        .times(1)
        .returning(|| Box::pin(async { Err(anyhow!("DB Error")) }));

    let app = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/devices")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
