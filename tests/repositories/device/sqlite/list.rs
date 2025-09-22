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
async fn success_multiple_devices() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    let devices = vec![
        ("dev1", Some("AA:BB:CC:DD:EE:01"), "key1"),
        ("dev2", Some("AA:BB:CC:DD:EE:02"), "key2"),
        ("dev3", None, "key3"),
    ];

    for (id, mac, api_key) in &devices {
        sqlx::query!(
            "INSERT INTO devices (id, mac, api_key, images_json) VALUES (?, ?, ?, ?)",
            id,
            mac,
            api_key,
            "[]"
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    let list = repo.list().await.unwrap();

    assert_eq!(list.len(), 3);

    assert_eq!(list[0].id, "dev1");
    assert_eq!(list[1].id, "dev2");
    assert_eq!(list[2].id, "dev3");
}

#[tokio::test]
async fn success_empty_table() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    let list = repo.list().await.unwrap();
    assert!(list.is_empty());
}
