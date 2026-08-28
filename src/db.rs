use sqlx::{Row, SqlitePool};

pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    for statement in include_str!("../migrations/0001_init.sql").split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement).execute(pool).await?;
        }
    }
    Ok(())
}

pub async fn ensure_instance_id(
    pool: &SqlitePool,
    candidate: &str,
) -> Result<&'static str, sqlx::Error> {
    let result = sqlx::query("INSERT OR IGNORE INTO app_meta(key, value) VALUES('instance_id', ?)")
        .bind(candidate)
        .execute(pool)
        .await?;
    Ok(if result.rows_affected() == 1 {
        "generated"
    } else {
        "existing"
    })
}

pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub async fn expire_due(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let now = now();
    let mut tx = pool.begin().await?;
    let due = sqlx::query(
        "SELECT id, inventory_id, quantity FROM holds WHERE status = 'active' AND expires_at <= ?",
    )
    .bind(now)
    .fetch_all(&mut *tx)
    .await?;
    let mut count = 0;
    for row in due {
        let id: String = row.get("id");
        let result = sqlx::query("UPDATE holds SET status = 'expired', resolved_at = ?, resolved_by = 'Clock' WHERE id = ? AND status = 'active'")
            .bind(now)
            .bind(&id)
            .execute(&mut *tx)
            .await?;
        if result.rows_affected() == 1 {
            let details = serde_json::json!({
                "inventory_id": row.get::<i64, _>("inventory_id"),
                "quantity": row.get::<i64, _>("quantity")
            });
            sqlx::query("INSERT INTO audit_log(event, entity_type, entity_id, actor, details_json, created_at) VALUES('hold.expired', 'hold', ?, 'Clock', ?, ?)")
                .bind(&id)
                .bind(details.to_string())
                .bind(now)
                .execute(&mut *tx)
                .await?;
            count += 1;
        }
    }
    tx.commit().await?;
    Ok(count)
}
