use sqlx::{Row, SqlitePool};

pub async fn migrate(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(include_str!("../migrations/0001_init.sql"))
        .execute(pool)
        .await?;
    let columns = sqlx::query("PRAGMA table_info(sessions)")
        .fetch_all(pool)
        .await?;
    let has_role = columns
        .iter()
        .any(|row| row.get::<String, _>("name") == "role");
    if !has_role {
        sqlx::query("ALTER TABLE sessions ADD COLUMN role TEXT NOT NULL DEFAULT 'supervisor'")
            .execute(pool)
            .await?;
    }
    let settings_columns = sqlx::query("PRAGMA table_info(settings)")
        .fetch_all(pool)
        .await?;
    let has_retention = settings_columns
        .iter()
        .any(|row| row.get::<String, _>("name") == "retention_days");
    if !has_retention {
        sqlx::query("ALTER TABLE settings ADD COLUMN retention_days INTEGER NOT NULL DEFAULT 90")
            .execute(pool)
            .await?;
    }
    Ok(())
}

pub async fn prepare_schema(pool: &SqlitePool) -> Result<&'static str, sqlx::Error> {
    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name IN ('settings','inventory','holds','audit_log','sessions','app_meta')",
    )
    .fetch_one(pool)
    .await?;
    if table_count == 6 {
        // Schema additions are intentionally idempotent so a durable /data
        // database is upgraded in place rather than replaced on a release.
        migrate(pool).await?;
        Ok("existing/upgraded")
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
    redact_retained_hold_details(pool).await?;
    Ok(count)
}

/// Retention never edits the append-only audit ledger. Instead it removes the
/// customer reference, note, and free-form operator name from resolved hold
/// records after the supervisor-selected period. New audit events deliberately
/// omit those values, so the immutable ledger remains useful without retaining
/// new customer content indefinitely.
pub async fn redact_retained_hold_details(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let Some(days) =
        sqlx::query_scalar::<_, i64>("SELECT retention_days FROM settings WHERE singleton = 1")
            .fetch_optional(pool)
            .await?
    else {
        return Ok(0);
    };
    let cutoff = now() - days * 86_400;
    let result = sqlx::query(
        "UPDATE holds SET customer = 'Removed after retention', order_note = '', operator_name = 'Removed after retention'
         WHERE status != 'active' AND resolved_at IS NOT NULL AND resolved_at <= ?
           AND (customer != 'Removed after retention' OR order_note != '' OR operator_name != 'Removed after retention')",
    ).bind(cutoff).execute(pool).await?;
    Ok(result.rows_affected())
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
        assert_eq!(
            prepare_schema(&read_only).await.unwrap(),
            "existing/upgraded"
        );
        assert_eq!(
            ensure_instance_id(&read_only, "must-not-write")
                .await
                .unwrap(),
            "existing"
        );
    }

    // @claim:retention-redaction
    #[tokio::test]
    async fn claim_retention_redaction_removes_personal_fields_but_keeps_audit_rows() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        migrate(&pool).await.unwrap();
        sqlx::query("INSERT INTO settings(singleton,location_name,supervisor_pin_hash,retention_days,created_at) VALUES(1,'Retention test','hash',30,0)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('RET-1','Retained part',1,0,0)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO holds(id,inventory_id,quantity,customer,order_note,operator_name,status,created_at,expires_at,resolved_at,resolved_by) VALUES('old',1,1,'Old customer','Private note','Old operator','released',1,2,3,'Supervisor')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO audit_log(event,entity_type,entity_id,actor,details_json,created_at) VALUES('hold.released','hold','old','oid','{}',3)")
            .execute(&pool).await.unwrap();
        assert_eq!(redact_retained_hold_details(&pool).await.unwrap(), 1);
        let values: (String, String, String) =
            sqlx::query_as("SELECT customer,order_note,operator_name FROM holds WHERE id='old'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(
            values,
            (
                "Removed after retention".into(),
                "".into(),
                "Removed after retention".into()
            )
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM audit_log")
                .fetch_one(&pool)
                .await
                .unwrap(),
            1
        );
    }
}
