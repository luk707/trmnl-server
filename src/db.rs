use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use tracing::info;

pub async fn connect(path: &str) -> anyhow::Result<SqlitePool> {
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true),
    )
    .await?;

    Ok(pool)
}

pub async fn apply_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    let before_count: i64 = match sqlx::query_scalar!("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
    {
        Ok(count) => count,
        Err(sqlx::Error::Database(_)) => 0,
        Err(e) => return Err(e.into()),
    };

    sqlx::migrate!("./migrations").run(pool).await?;

    let after_count: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await?;

    let limit = after_count - before_count;

    let new_migrations = sqlx::query!(
        "SELECT version, description FROM _sqlx_migrations ORDER BY version DESC LIMIT ?",
        limit
    )
    .fetch_all(pool)
    .await?;

    for m in new_migrations {
        info!(
            msg = "Applied database migration",
            version = %m.version.map(|v| v.to_string()).unwrap_or_default(),
            description = %m.description,
        );
    }

    Ok(())
}
