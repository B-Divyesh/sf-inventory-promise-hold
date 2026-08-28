use sqlx::{Row, SqlitePool};

pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn prepare_schema(pool: &SqlitePool) -> Result<&'static str, sqlx::Error> {
    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('settings','inventory','holds','audit_log','sessions','app_meta')",
    )
    .fetch_one(pool)
    .await?;
    if table_count == 6 {
        Ok("existing")
    } else {
        migrate(pool).await?;
        Ok("migrated")
    }
}

pub async fn ensure_instance_id(
    pool: &SqlitePool,
    candidate: &str,
) -> Result<&'static str, sqlx::Error> {
    if sqlx::query("SELECT 1 FROM app_meta WHERE key = 'instance_id'")
        .fetch_optional(pool)
        .await?
        .is_some()
    {
        return Ok("existing");
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

    async fn open(path: &std::path::Path) -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(path)
                    .create_if_missing(true)
                    .journal_mode(SqliteJournalMode::Wal),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn location_inventory_and_audit_survive_pool_restart() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("persistent.db");
        let pool = open(&path).await;
        migrate(&pool).await.unwrap();
        ensure_instance_id(&pool, "stable-instance").await.unwrap();
        sqlx::query("INSERT INTO settings(singleton,location_name,supervisor_pin_hash,created_at) VALUES(1,'Persistent stockroom','hash',1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('KEEP-1','Retained item',7,1,1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO audit_log(event,entity_type,entity_id,actor,details_json,created_at) VALUES('inventory.created','inventory','1','Supervisor','{}',1)")
            .execute(&pool).await.unwrap();
        pool.close().await;

        let reopened = open(&path).await;
        migrate(&reopened).await.unwrap();
        assert_eq!(
            sqlx::query_scalar::<_, String>("SELECT location_name FROM settings WHERE singleton=1")
                .fetch_one(&reopened)
                .await
                .unwrap(),
            "Persistent stockroom"
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT on_hand FROM inventory WHERE sku='KEEP-1'")
                .fetch_one(&reopened)
                .await
                .unwrap(),
            7
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM audit_log")
                .fetch_one(&reopened)
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            ensure_instance_id(&reopened, "replacement").await.unwrap(),
            "existing"
        );
        reopened.close().await;

        let read_only = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(SqliteConnectOptions::new().filename(&path).read_only(true))
            .await
            .unwrap();
        assert_eq!(prepare_schema(&read_only).await.unwrap(), "existing");
        assert_eq!(
            ensure_instance_id(&read_only, "must-not-write")
                .await
                .unwrap(),
            "existing"
        );
    }
}
