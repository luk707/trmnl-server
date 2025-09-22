use anyhow::anyhow;
use axum::{
    body::{Body, to_bytes},
    extract::Extension,
    http::{Request, StatusCode},
};
use mockall::predicate;
use std::sync::Arc;
use tower::ServiceExt;
use trmnl_server::{
    app::App,
    config::AppSettings,
    headers::{HEADER_ACCESS_TOKEN, HEADER_BATTERY_VOLTAGE, HEADER_FW_VERSION, HEADER_RSSI},
    layers::device::DeviceRepoLayer,
    models::DisplayResponse,
    repositories::device::MockDeviceRepository,
};

fn test_settings() -> AppSettings {
    AppSettings {
        setup_logo_url: "https://example.com/logo.png".to_string(),
    }
}

#[tokio::test]
async fn success_found() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_get_by_api_key()
        .with(predicate::eq("valid-token"))
        .times(1)
        .returning(|_token| {
            Box::pin(async {
                Ok(Some(trmnl_server::models::Device {
                    id: "dev123".to_string(),
                    mac: None,
                    _api_key: "valid-token".to_string(),
                    rssi: None,
                    battery_voltage: None,
                    fw_version: None,
                    refresh_rate: None,
                    images: vec![],
                }))
            })
        });

    mock_repo
        .expect_update_status()
        .times(1)
        .returning(|_, _, _, _, _| Box::pin(async { Ok(()) }));

    let app = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(test_settings()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/display")
                .header(&HEADER_ACCESS_TOKEN, "valid-token")
                .header(&HEADER_RSSI, "-70")
                .header(&HEADER_FW_VERSION, "1.0.0")
                .header(&HEADER_BATTERY_VOLTAGE, "3.7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: DisplayResponse = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json.status, 0);
    assert_eq!(json.image_url, "https://example.com/logo.png");
    assert_eq!(json.update_firmware, false);
}

#[tokio::test]
async fn success_not_found() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_get_by_api_key()
        .with(predicate::eq("invalid-token"))
        .times(1)
        .returning(|_token| Box::pin(async { Ok(None) }));

    let app = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(test_settings()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/display")
                .header(&HEADER_ACCESS_TOKEN, "invalid-token")
                .header(&HEADER_RSSI, "-70")
                .header(&HEADER_FW_VERSION, "1.0.0")
                .header(&HEADER_BATTERY_VOLTAGE, "3.7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK); // Handler always returns 200
    let body_bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: DisplayResponse = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(json.status, 500);
}

#[tokio::test]
async fn error_get_by_api_key() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_get_by_api_key()
        .with(predicate::eq("valid-token"))
        .times(1)
        .returning(|_token| Box::pin(async { Err(anyhow!("DB Error")) }));

    mock_repo.expect_update_status().times(0);

    let app = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(test_settings()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/display")
                .header(&HEADER_ACCESS_TOKEN, "valid-token")
                .header(&HEADER_RSSI, "-70")
                .header(&HEADER_FW_VERSION, "1.0.0")
                .header(&HEADER_BATTERY_VOLTAGE, "3.7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn error_update_status() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_get_by_api_key()
        .with(predicate::eq("valid-token"))
        .times(1)
        .returning(|_token| {
            Box::pin(async {
                Ok(Some(trmnl_server::models::Device {
                    id: "dev123".to_string(),
                    mac: None,
                    _api_key: "valid-token".to_string(),
                    rssi: None,
                    battery_voltage: None,
                    fw_version: None,
                    refresh_rate: None,
                    images: vec![],
                }))
            })
        });

    mock_repo
        .expect_update_status()
        .times(1)
        .returning(|_, _, _, _, _| Box::pin(async { Err(anyhow!("DB Error")) }));

    let app = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .layer(Extension(test_settings()));

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/display")
                .header(&HEADER_ACCESS_TOKEN, "valid-token")
                .header(&HEADER_RSSI, "-70")
                .header(&HEADER_FW_VERSION, "1.0.0")
                .header(&HEADER_BATTERY_VOLTAGE, "3.7")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
