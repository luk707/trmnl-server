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

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < f32::EPSILON.into()
}

#[tokio::test]
async fn success_update_all_fields() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    sqlx::query!(
        "INSERT INTO devices (id, mac, api_key, rssi, battery_voltage, fw_version, refresh_rate, images_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        "dev123",
        Some("AA:BB:CC:DD:EE:FF"),
        "apikey123",
        Some(-50),
        Some(3.7),
        Some("1.0.0"),
        Some(60),
        "[]"
    )
    .execute(&pool)
    .await
    .unwrap();

    repo.update_status("dev123", Some(-70), Some(3.9), Some("1.1.0"), Some(30))
        .await
        .unwrap();

    let record = sqlx::query!(
        "SELECT rssi, battery_voltage, fw_version, refresh_rate FROM devices WHERE id = ?",
        "dev123"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(record.rssi, Some(-70));
    assert!(approx_eq(record.battery_voltage.unwrap(), 3.9));
    assert_eq!(record.fw_version.as_deref(), Some("1.1.0"));
    assert_eq!(record.refresh_rate, Some(30));
}

#[tokio::test]
async fn success_partial_update() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    sqlx::query!(
        "INSERT INTO devices (id, mac, api_key, rssi, battery_voltage, fw_version, refresh_rate, images_json)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        "dev123",
        Some("AA:BB:CC:DD:EE:FF"),
        "apikey123",
        Some(-50),
        Some(3.7),
        Some("1.0.0"),
        Some(60),
        "[]"
    )
    .execute(&pool)
    .await
    .unwrap();

    repo.update_status("dev123", Some(-60), None, Some("1.0.1"), None)
        .await
        .unwrap();

    let record = sqlx::query!(
        "SELECT rssi, battery_voltage, fw_version, refresh_rate FROM devices WHERE id = ?",
        "dev123"
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(record.rssi, Some(-60));
    assert!(approx_eq(record.battery_voltage.unwrap(), 3.7));
    assert_eq!(record.fw_version.as_deref(), Some("1.0.1"));
    assert_eq!(record.refresh_rate, Some(60));
}

#[tokio::test]
async fn success_update_nonexistent_device() {
    let pool = connect().await.unwrap();
    let repo = SqliteDeviceRepo::new(Arc::new(pool.clone()));

    repo.update_status("nonexistent", Some(-70), Some(3.9), Some("1.1.0"), Some(30))
        .await
        .unwrap();

    let count = sqlx::query!("SELECT COUNT(*) as count FROM devices")
        .fetch_one(&pool)
        .await
        .unwrap()
        .count;
    assert_eq!(count, 0);
}
