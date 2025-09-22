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
async fn success_create_with_mac() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    let result = repo
        .create("dev123", Some("AA:BB:CC:DD:EE:FF"), "apikey123")
        .await;
    assert!(result.is_ok());

    let exists = sqlx::query!(
        "SELECT id, mac, api_key FROM devices WHERE id = ?",
        "dev123"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(exists.id, "dev123");
    assert_eq!(exists.mac.unwrap(), "AA:BB:CC:DD:EE:FF");
    assert_eq!(exists.api_key, "apikey123");
}

#[tokio::test]
async fn success_create_without_mac() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    let result = repo.create("dev456", None, "apikey456").await;
    assert!(result.is_ok());

    let exists = sqlx::query!(
        "SELECT id, mac, api_key FROM devices WHERE id = ?",
        "dev456"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(exists.id, "dev456");
    assert!(exists.mac.is_none());
    assert_eq!(exists.api_key, "apikey456");
}

#[tokio::test]
async fn error_duplicate_id() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    repo.create("dev789", Some("AA:BB:CC:DD:EE:01"), "apikey789")
        .await
        .unwrap();

    let result = repo
        .create("dev789", Some("AA:BB:CC:DD:EE:02"), "apikey999")
        .await;
    assert!(result.is_err());
}
