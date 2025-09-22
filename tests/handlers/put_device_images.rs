use std::sync::Arc;

use anyhow::anyhow;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use mockall::predicate;
use serde_json::Value;
use tower::ServiceExt;
use trmnl_server::{
    app::App, layers::device::DeviceRepoLayer, repositories::device::MockDeviceRepository,
};

#[tokio::test]
async fn success() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_update_images()
        .with(
            predicate::eq("dev123".to_string()),
            predicate::eq(vec!["one.jpg".to_string(), "two.jpg".to_string()]),
        )
        .times(1)
        .returning(|_, _| Box::pin(async { Ok(()) }));

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/devices/dev123/images")
                .header("content-type", "application/json")
                .body(Body::from(r#"["one.jpg","two.jpg"]"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json, serde_json::json!(["one.jpg", "two.jpg"]));
}

#[tokio::test]
async fn error() {
    let mut mock_repo = MockDeviceRepository::new();

    mock_repo
        .expect_update_images()
        .with(
            predicate::eq("dev123".to_string()),
            predicate::eq(vec!["one.jpg".to_string(), "two.jpg".to_string()]),
        )
        .times(1)
        .returning(|_, _| Box::pin(async { Err(anyhow!("DB Error")) }));

    let response = App::new()
        .router()
        .layer(DeviceRepoLayer(Arc::new(mock_repo)))
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/devices/dev123/images")
                .header("content-type", "application/json")
                .body(Body::from(r#"["one.jpg","two.jpg"]"#))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
