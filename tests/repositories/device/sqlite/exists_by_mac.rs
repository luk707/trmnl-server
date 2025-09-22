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

    apply_migrations(&pool).await.unwrap();

    Ok(pool)
}

#[tokio::test]
async fn success_true() {
    let pool = connect().await.unwrap();

    sqlx::query!(
        "INSERT INTO devices (id, mac, api_key, images_json) VALUES (?, ?, ?, ?)",
        "dev123",
        "AA:BB:CC:DD:EE:FF",
        "apikey123",
        "[]"
    )
    .execute(&pool)
    .await
    .unwrap();

    let repo = SqliteDeviceRepo::new(Arc::new(pool));

    let exists = repo.exists_by_mac("AA:BB:CC:DD:EE:FF").await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn success_false() {
    let pool = connect().await.unwrap();

    let repo = SqliteDeviceRepo::new(Arc::new(pool));

    let exists = repo.exists_by_mac("11:22:33:44:55:66").await.unwrap();
    assert!(!exists);
}
