use std::sync::Arc;

use sqlx::SqlitePool;
use trmnl_server::{
    db::apply_migrations,
    repositories::device::{DeviceRepository, SqliteDeviceRepo},
};

async fn connect() -> anyhow::Result<SqlitePool> {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    apply_migrations(&pool).await?;
    Ok(pool)
}

#[tokio::test]
async fn success_found() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    sqlx::query!(
        r#"
        INSERT INTO devices (id, mac, api_key, images_json)
        VALUES (?, ?, ?, ?)
        "#,
        "dev123",
        Some("AA:BB:CC:DD:EE:FF"),
        "apikey123",
        "[]"
    )
    .execute(&pool)
    .await
    .unwrap();

    let device = repo.get_by_api_key("apikey123").await.unwrap().unwrap();

    assert_eq!(device.id, "dev123");
    assert_eq!(device.mac.unwrap(), "AA:BB:CC:DD:EE:FF");
    assert_eq!(device._api_key, "apikey123");
    assert!(device.images.is_empty());
}

#[tokio::test]
async fn success_not_found() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    let device = repo.get_by_api_key("nonexistent").await.unwrap();
    assert!(device.is_none());
}
