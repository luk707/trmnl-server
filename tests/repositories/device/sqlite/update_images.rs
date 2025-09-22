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
async fn success_update_existing_device() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    // Insert a device
    sqlx::query!(
        "INSERT INTO devices (id, mac, api_key, images_json) VALUES (?, ?, ?, ?)",
        "dev123",
        Some("AA:BB:CC:DD:EE:FF"),
        "apikey123",
        "[]"
    )
    .execute(&pool)
    .await
    .unwrap();

    let images = vec!["image1.jpg".to_string(), "image2.jpg".to_string()];

    let result = repo.update_images("dev123", &images).await;
    assert!(result.is_ok());

    // Verify the images were updated
    let record = sqlx::query!("SELECT images_json FROM devices WHERE id = ?", "dev123")
        .fetch_one(&pool)
        .await
        .unwrap();

    let stored_images: Vec<String> = serde_json::from_str(&record.images_json).unwrap();
    assert_eq!(stored_images, images);
}

#[tokio::test]
async fn success_update_nonexistent_device() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    // Attempt to update images for a device that doesn't exist
    let images = vec!["image1.jpg".to_string()];
    let result = repo.update_images("nonexistent", &images).await;

    // SQL UPDATE succeeds even if no rows match
    assert!(result.is_ok());

    // Verify no rows exist
    let count = sqlx::query!("SELECT COUNT(*) as count FROM devices")
        .fetch_one(&pool)
        .await
        .unwrap()
        .count;
    assert_eq!(count, 0);
}
