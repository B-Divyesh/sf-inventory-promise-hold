use argon2::{
    password_hash::{
        rand_core::OsRng as PasswordOsRng, PasswordHash, PasswordHasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::db;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub build_sha: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error("The server could not complete that action. Try again.")]
    Internal(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/bootstrap", get(bootstrap))
        .route("/setup", post(setup))
        .route("/session", post(login).delete(logout))
        .route("/inventory", post(create_inventory))
        .route("/inventory/{id}", post(update_inventory))
        .route("/holds", post(create_hold))
        .route("/holds/{id}/resolve", post(resolve_hold))
        .route("/audit", get(audit))
        .route("/export.csv", get(export_csv))
}

pub async fn health(State(state): State<AppState>) -> Json<Value> {
    Json(json!({ "status": "ok", "build_sha": state.build_sha }))
}

#[derive(Serialize)]
struct Bootstrap {
    setup_required: bool,
    location_name: Option<String>,
    server_time: i64,
    inventory: Vec<InventoryView>,
    active_holds: Vec<HoldView>,
    recent_outcomes: Vec<HoldView>,
}

#[derive(Serialize)]
struct InventoryView {
    id: i64,
    sku: String,
    name: String,
    on_hand: i64,
    held: i64,
    available: i64,
}

#[derive(Serialize)]
struct HoldView {
    id: String,
    inventory_id: i64,
    sku: String,
    item_name: String,
    quantity: i64,
    customer: String,
    order_note: String,
    operator_name: String,
    status: String,
    created_at: i64,
    expires_at: i64,
    resolved_at: Option<i64>,
    resolved_by: Option<String>,
}

async fn bootstrap(State(state): State<AppState>) -> Result<Json<Bootstrap>, ApiError> {
    db::expire_due(&state.pool).await?;
    let setting = sqlx::query("SELECT location_name FROM settings WHERE singleton = 1")
        .fetch_optional(&state.pool)
        .await?;
    let inventory_rows = sqlx::query(
        "SELECT i.id, i.sku, i.name, i.on_hand, COALESCE(SUM(CASE WHEN h.status = 'active' THEN h.quantity ELSE 0 END), 0) AS held
         FROM inventory i LEFT JOIN holds h ON h.inventory_id = i.id
         GROUP BY i.id ORDER BY i.sku COLLATE NOCASE",
    )
    .fetch_all(&state.pool)
    .await?;
    let inventory = inventory_rows
        .into_iter()
        .map(|row| {
            let on_hand = row.get("on_hand");
            let held = row.get("held");
            InventoryView {
                id: row.get("id"),
                sku: row.get("sku"),
                name: row.get("name"),
                on_hand,
                held,
                available: on_hand - held,
            }
        })
        .collect();
    let active_holds = fetch_holds(&state.pool, "h.status = 'active'", 250).await?;
    let recent_outcomes = fetch_holds(&state.pool, "h.status != 'active'", 60).await?;
    Ok(Json(Bootstrap {
        setup_required: setting.is_none(),
        location_name: setting.map(|row| row.get("location_name")),
        server_time: db::now(),
        inventory,
        active_holds,
        recent_outcomes,
    }))
}

async fn fetch_holds(
    pool: &SqlitePool,
    predicate: &str,
    limit: i64,
) -> Result<Vec<HoldView>, sqlx::Error> {
    let query = format!(
        "SELECT h.*, i.sku, i.name AS item_name FROM holds h JOIN inventory i ON i.id = h.inventory_id WHERE {predicate} ORDER BY CASE WHEN h.status = 'active' THEN h.expires_at ELSE h.resolved_at END ASC LIMIT ?"
    );
    let rows = sqlx::query(&query).bind(limit).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|row| HoldView {
            id: row.get("id"),
            inventory_id: row.get("inventory_id"),
            sku: row.get("sku"),
            item_name: row.get("item_name"),
            quantity: row.get("quantity"),
            customer: row.get("customer"),
            order_note: row.get("order_note"),
            operator_name: row.get("operator_name"),
            status: row.get("status"),
            created_at: row.get("created_at"),
            expires_at: row.get("expires_at"),
            resolved_at: row.get("resolved_at"),
            resolved_by: row.get("resolved_by"),
        })
        .collect())
}

#[derive(Deserialize)]
struct SetupInput {
    location_name: String,
    pin: String,
}

#[derive(Serialize)]
struct SessionOutput {
    token: String,
    expires_at: i64,
}

async fn setup(
    State(state): State<AppState>,
    Json(input): Json<SetupInput>,
) -> Result<Json<SessionOutput>, ApiError> {
    let location = required_text(&input.location_name, "Location name", 80)?;
    validate_pin(&input.pin)?;
    let pin = input.pin.clone();
    let hash = tokio::task::spawn_blocking(move || {
        let salt = SaltString::generate(&mut PasswordOsRng);
        Argon2::default()
            .hash_password(pin.as_bytes(), &salt)
            .map(|v| v.to_string())
    })
    .await
    .map_err(|_| ApiError::BadRequest("Could not secure that PIN. Try again.".into()))?
    .map_err(|_| ApiError::BadRequest("Could not secure that PIN. Try again.".into()))?;
    let now = db::now();
    let mut tx = state.pool.begin().await?;
    if sqlx::query("SELECT 1 FROM settings WHERE singleton = 1")
        .fetch_optional(&mut *tx)
        .await?
        .is_some()
    {
        return Err(ApiError::Conflict(
            "This location is already set up.".into(),
        ));
    }
    sqlx::query("INSERT INTO settings(singleton, location_name, supervisor_pin_hash, created_at) VALUES(1, ?, ?, ?)")
        .bind(&location)
        .bind(hash)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    sqlx::query("INSERT INTO audit_log(event, entity_type, entity_id, actor, details_json, created_at) VALUES('location.setup', 'location', '1', 'Supervisor', ?, ?)")
        .bind(json!({ "location_name": location }).to_string())
        .bind(now)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(create_session(&state.pool).await?))
}

#[derive(Deserialize)]
struct LoginInput {
    pin: String,
}

async fn login(
    State(state): State<AppState>,
    Json(input): Json<LoginInput>,
) -> Result<Json<SessionOutput>, ApiError> {
    let row = sqlx::query("SELECT supervisor_pin_hash FROM settings WHERE singleton = 1")
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::Conflict("Set up this location first.".into()))?;
    let hash: String = row.get("supervisor_pin_hash");
    let pin = input.pin;
    let valid = tokio::task::spawn_blocking(move || {
        PasswordHash::new(&hash).ok().is_some_and(|parsed| {
            Argon2::default()
                .verify_password(pin.as_bytes(), &parsed)
                .is_ok()
        })
    })
    .await
    .unwrap_or(false);
    if !valid {
        tokio::time::sleep(std::time::Duration::from_millis(450)).await;
        return Err(ApiError::Unauthorized(
            "That supervisor PIN is not correct.".into(),
        ));
    }
    Ok(Json(create_session(&state.pool).await?))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode, ApiError> {
    let token = bearer(&headers)?;
    sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
        .bind(token_hash(token))
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_session(pool: &SqlitePool) -> Result<SessionOutput, ApiError> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = hex(&bytes);
    let now = db::now();
    let expires_at = now + 12 * 60 * 60;
    sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
        .bind(now)
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO sessions(token_hash, expires_at, created_at) VALUES(?, ?, ?)")
        .bind(token_hash(&token))
        .bind(expires_at)
        .bind(now)
        .execute(pool)
        .await?;
    Ok(SessionOutput { token, expires_at })
}

async fn require_supervisor(pool: &SqlitePool, headers: &HeaderMap) -> Result<(), ApiError> {
    let token = bearer(headers)?;
    let valid = sqlx::query("SELECT 1 FROM sessions WHERE token_hash = ? AND expires_at > ?")
        .bind(token_hash(token))
        .bind(db::now())
        .fetch_optional(pool)
        .await?
        .is_some();
    if valid {
        Ok(())
    } else {
        Err(ApiError::Unauthorized(
            "Supervisor access has expired. Unlock it again.".into(),
        ))
    }
}

#[derive(Deserialize)]
struct InventoryInput {
    sku: String,
    name: String,
    on_hand: i64,
}

async fn create_inventory(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<InventoryInput>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    require_supervisor(&state.pool, &headers).await?;
    let sku = normalize_sku(&input.sku)?;
    let name = required_text(&input.name, "Item name", 120)?;
    validate_stock(input.on_hand)?;
    let now = db::now();
    let result = sqlx::query(
        "INSERT INTO inventory(sku, name, on_hand, created_at, updated_at) VALUES(?, ?, ?, ?, ?)",
    )
    .bind(&sku)
    .bind(&name)
    .bind(input.on_hand)
    .bind(now)
    .bind(now)
    .execute(&state.pool)
    .await;
    let id = match result {
        Ok(result) => result.last_insert_rowid(),
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            return Err(ApiError::Conflict(format!("SKU {sku} already exists.")))
        }
        Err(error) => return Err(error.into()),
    };
    audit_event(
        &state.pool,
        "inventory.created",
        "inventory",
        &id.to_string(),
        "Supervisor",
        json!({"sku": sku, "name": name, "on_hand": input.on_hand}),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn update_inventory(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    headers: HeaderMap,
    Json(input): Json<InventoryInput>,
) -> Result<Json<Value>, ApiError> {
    require_supervisor(&state.pool, &headers).await?;
    let sku = normalize_sku(&input.sku)?;
    let name = required_text(&input.name, "Item name", 120)?;
    validate_stock(input.on_hand)?;
    db::expire_due(&state.pool).await?;
    let held: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(quantity), 0) FROM holds WHERE inventory_id = ? AND status = 'active'",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    if input.on_hand < held {
        return Err(ApiError::Conflict(format!(
            "{held} units are actively held. On-hand stock cannot be lower than that."
        )));
    }
    let result = sqlx::query(
        "UPDATE inventory SET sku = ?, name = ?, on_hand = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&sku)
    .bind(&name)
    .bind(input.on_hand)
    .bind(db::now())
    .bind(id)
    .execute(&state.pool)
    .await;
    let result = match result {
        Ok(result) => result,
        Err(sqlx::Error::Database(error)) if error.is_unique_violation() => {
            return Err(ApiError::Conflict(format!("SKU {sku} already exists.")))
        }
        Err(error) => return Err(error.into()),
    };
    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound(
            "That inventory item no longer exists.".into(),
        ));
    }
    audit_event(
        &state.pool,
        "inventory.updated",
        "inventory",
        &id.to_string(),
        "Supervisor",
        json!({"sku": sku, "name": name, "on_hand": input.on_hand}),
    )
    .await?;
    Ok(Json(json!({ "updated": true })))
}

#[derive(Deserialize)]
struct HoldInput {
    inventory_id: i64,
    quantity: i64,
    customer: String,
    order_note: Option<String>,
    operator_name: String,
    duration_minutes: i64,
}

async fn create_hold(
    State(state): State<AppState>,
    Json(input): Json<HoldInput>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if input.quantity <= 0 || input.quantity > 1_000_000 {
        return Err(ApiError::BadRequest(
            "Quantity must be between 1 and 1,000,000.".into(),
        ));
    }
    if !(5..=480).contains(&input.duration_minutes) {
        return Err(ApiError::BadRequest(
            "Hold time must be between 5 minutes and 8 hours.".into(),
        ));
    }
    let customer = required_text(&input.customer, "Customer", 120)?;
    let operator = required_text(&input.operator_name, "Operator name", 80)?;
    let note = limited_text(input.order_note.as_deref().unwrap_or(""), "Order note", 300)?;
    let now = db::now();
    let expires_at = now + input.duration_minutes * 60;
    let id = Uuid::new_v4().to_string();
    let mut conn = state.pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    let outcome = create_hold_locked(
        &mut conn, &input, &id, &customer, &operator, &note, now, expires_at,
    )
    .await;
    match outcome {
        Ok(available_after) => {
            sqlx::query("COMMIT").execute(&mut *conn).await?;
            Ok((
                StatusCode::CREATED,
                Json(
                    json!({ "id": id, "expires_at": expires_at, "available_after": available_after }),
                ),
            ))
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            Err(error)
        }
    }
}

async fn create_hold_locked(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    input: &HoldInput,
    id: &str,
    customer: &str,
    operator: &str,
    note: &str,
    now: i64,
    expires_at: i64,
) -> Result<i64, ApiError> {
    expire_due_locked(conn, now).await?;
    let item = sqlx::query("SELECT sku, name, on_hand FROM inventory WHERE id = ?")
        .bind(input.inventory_id)
        .fetch_optional(&mut **conn)
        .await?
        .ok_or_else(|| ApiError::NotFound("That inventory item no longer exists.".into()))?;
    let held: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(quantity), 0) FROM holds WHERE inventory_id = ? AND status = 'active'",
    )
    .bind(input.inventory_id)
    .fetch_one(&mut **conn)
    .await?;
    let on_hand: i64 = item.get("on_hand");
    let available = on_hand - held;
    if input.quantity > available {
        return Err(ApiError::Conflict(format!(
            "Only {available} units are available now. Refresh and adjust the hold."
        )));
    }
    sqlx::query("INSERT INTO holds(id, inventory_id, quantity, customer, order_note, operator_name, status, created_at, expires_at) VALUES(?, ?, ?, ?, ?, ?, 'active', ?, ?)")
        .bind(id).bind(input.inventory_id).bind(input.quantity).bind(customer).bind(note).bind(operator).bind(now).bind(expires_at)
        .execute(&mut **conn).await?;
    let details = json!({"sku": item.get::<String, _>("sku"), "item_name": item.get::<String, _>("name"), "quantity": input.quantity, "customer": customer, "expires_at": expires_at});
    sqlx::query("INSERT INTO audit_log(event, entity_type, entity_id, actor, details_json, created_at) VALUES('hold.created', 'hold', ?, ?, ?, ?)")
        .bind(id).bind(operator).bind(details.to_string()).bind(now).execute(&mut **conn).await?;
    Ok(available - input.quantity)
}

async fn expire_due_locked(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    now: i64,
) -> Result<(), sqlx::Error> {
    let due = sqlx::query(
        "SELECT id, inventory_id, quantity FROM holds WHERE status = 'active' AND expires_at <= ?",
    )
    .bind(now)
    .fetch_all(&mut **conn)
    .await?;
    for row in due {
        let id: String = row.get("id");
        let result = sqlx::query("UPDATE holds SET status = 'expired', resolved_at = ?, resolved_by = 'Clock' WHERE id = ? AND status = 'active'")
            .bind(now).bind(&id).execute(&mut **conn).await?;
        if result.rows_affected() == 1 {
            sqlx::query("INSERT INTO audit_log(event, entity_type, entity_id, actor, details_json, created_at) VALUES('hold.expired', 'hold', ?, 'Clock', ?, ?)")
                .bind(&id).bind(json!({"inventory_id": row.get::<i64,_>("inventory_id"), "quantity": row.get::<i64,_>("quantity")}).to_string())
                .bind(now).execute(&mut **conn).await?;
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct ResolveInput {
    action: String,
    actor: String,
}

async fn resolve_hold(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<ResolveInput>,
) -> Result<Json<Value>, ApiError> {
    require_supervisor(&state.pool, &headers).await?;
    let actor = required_text(&input.actor, "Supervisor name", 80)?;
    if input.action != "convert" && input.action != "release" {
        return Err(ApiError::BadRequest(
            "Action must be convert or release.".into(),
        ));
    }
    let now = db::now();
    let mut conn = state.pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    let outcome = resolve_hold_locked(&mut conn, &id, &input.action, &actor, now).await;
    match outcome {
        Ok(()) => {
            sqlx::query("COMMIT").execute(&mut *conn).await?;
            Ok(Json(
                json!({ "status": if input.action == "convert" { "converted" } else { "released" } }),
            ))
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            Err(error)
        }
    }
}

async fn resolve_hold_locked(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    id: &str,
    action: &str,
    actor: &str,
    now: i64,
) -> Result<(), ApiError> {
    expire_due_locked(conn, now).await?;
    let hold = sqlx::query("SELECT inventory_id, quantity, status FROM holds WHERE id = ?")
        .bind(id)
        .fetch_optional(&mut **conn)
        .await?
        .ok_or_else(|| ApiError::NotFound("That hold no longer exists.".into()))?;
    let status: String = hold.get("status");
    if status != "active" {
        return Err(ApiError::Conflict(format!(
            "That hold is already {status}."
        )));
    }
    let inventory_id: i64 = hold.get("inventory_id");
    let quantity: i64 = hold.get("quantity");
    let next_status = if action == "convert" {
        "converted"
    } else {
        "released"
    };
    if action == "convert" {
        let result = sqlx::query("UPDATE inventory SET on_hand = on_hand - ?, updated_at = ? WHERE id = ? AND on_hand >= ?")
            .bind(quantity).bind(now).bind(inventory_id).bind(quantity).execute(&mut **conn).await?;
        if result.rows_affected() != 1 {
            return Err(ApiError::Conflict(
                "Stock changed and this hold can no longer be converted.".into(),
            ));
        }
    }
    sqlx::query("UPDATE holds SET status = ?, resolved_at = ?, resolved_by = ? WHERE id = ? AND status = 'active'")
        .bind(next_status).bind(now).bind(actor).bind(id).execute(&mut **conn).await?;
    sqlx::query("INSERT INTO audit_log(event, entity_type, entity_id, actor, details_json, created_at) VALUES(?, 'hold', ?, ?, ?, ?)")
        .bind(format!("hold.{next_status}")).bind(id).bind(actor)
        .bind(json!({"inventory_id": inventory_id, "quantity": quantity}).to_string()).bind(now)
        .execute(&mut **conn).await?;
    Ok(())
}

async fn audit(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    require_supervisor(&state.pool, &headers).await?;
    let rows = sqlx::query("SELECT id, event, entity_type, entity_id, actor, details_json, created_at FROM audit_log ORDER BY id DESC LIMIT 500")
        .fetch_all(&state.pool).await?;
    let entries: Vec<Value> = rows.into_iter().map(|row| json!({
        "id": row.get::<i64,_>("id"), "event": row.get::<String,_>("event"), "entity_type": row.get::<String,_>("entity_type"),
        "entity_id": row.get::<String,_>("entity_id"), "actor": row.get::<String,_>("actor"),
        "details": serde_json::from_str::<Value>(&row.get::<String,_>("details_json")).unwrap_or(Value::Null), "created_at": row.get::<i64,_>("created_at")
    })).collect();
    Ok(Json(json!({ "entries": entries })))
}

async fn export_csv(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    require_supervisor(&state.pool, &headers).await?;
    db::expire_due(&state.pool).await?;
    let rows = sqlx::query("SELECT h.id, i.sku, i.name, h.quantity, h.customer, h.order_note, h.operator_name, h.status, h.created_at, h.expires_at, h.resolved_at, h.resolved_by FROM holds h JOIN inventory i ON i.id = h.inventory_id ORDER BY h.created_at DESC")
        .fetch_all(&state.pool).await?;
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record([
            "hold_id",
            "sku",
            "item",
            "quantity",
            "customer",
            "order_note",
            "operator",
            "outcome",
            "created_unix",
            "expires_unix",
            "resolved_unix",
            "resolved_by",
        ])
        .map_err(|_| ApiError::Internal(sqlx::Error::Protocol("csv header".into())))?;
    for row in rows {
        writer
            .write_record(&[
                row.get::<String, _>("id"),
                row.get::<String, _>("sku"),
                row.get::<String, _>("name"),
                row.get::<i64, _>("quantity").to_string(),
                row.get::<String, _>("customer"),
                row.get::<String, _>("order_note"),
                row.get::<String, _>("operator_name"),
                row.get::<String, _>("status"),
                row.get::<i64, _>("created_at").to_string(),
                row.get::<i64, _>("expires_at").to_string(),
                row.get::<Option<i64>, _>("resolved_at")
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                row.get::<Option<String>, _>("resolved_by")
                    .unwrap_or_default(),
            ])
            .map_err(|_| ApiError::Internal(sqlx::Error::Protocol("csv row".into())))?;
    }
    let bytes = writer
        .into_inner()
        .map_err(|_| ApiError::Internal(sqlx::Error::Protocol("csv output".into())))?;
    let mut response = Response::new(Body::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=stock-promise-holds.csv"),
    );
    Ok(response)
}

async fn audit_event(
    pool: &SqlitePool,
    event: &str,
    entity_type: &str,
    entity_id: &str,
    actor: &str,
    details: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO audit_log(event, entity_type, entity_id, actor, details_json, created_at) VALUES(?, ?, ?, ?, ?, ?)")
        .bind(event).bind(entity_type).bind(entity_id).bind(actor).bind(details.to_string()).bind(db::now()).execute(pool).await?;
    Ok(())
}

fn bearer(headers: &HeaderMap) -> Result<&str, ApiError> {
    headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .filter(|v| !v.is_empty())
        .ok_or_else(|| ApiError::Unauthorized("Unlock supervisor access to continue.".into()))
}

fn token_hash(token: &str) -> String {
    hex(&Sha256::digest(token.as_bytes()))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn validate_pin(pin: &str) -> Result<(), ApiError> {
    if (6..=12).contains(&pin.len()) && pin.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "Supervisor PIN must be 6–12 digits.".into(),
        ))
    }
}

fn normalize_sku(value: &str) -> Result<String, ApiError> {
    let value = required_text(value, "SKU", 48)?.to_uppercase();
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/'))
    {
        Ok(value)
    } else {
        Err(ApiError::BadRequest(
            "SKU may use letters, numbers, dashes, underscores, periods, and slashes.".into(),
        ))
    }
}

fn required_text(value: &str, label: &str, max: usize) -> Result<String, ApiError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ApiError::BadRequest(format!("{label} is required.")));
    }
    limited_text(value, label, max)
}

fn limited_text(value: &str, label: &str, max: usize) -> Result<String, ApiError> {
    let value = value.trim();
    if value.chars().count() > max {
        Err(ApiError::BadRequest(format!(
            "{label} must be {max} characters or fewer."
        )))
    } else {
        Ok(value.to_string())
    }
}

fn validate_stock(value: i64) -> Result<(), ApiError> {
    if (0..=100_000_000).contains(&value) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "On-hand stock must be between 0 and 100,000,000.".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn state() -> AppState {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::db::migrate(&pool).await.unwrap();
        AppState {
            pool,
            build_sha: "test".into(),
        }
    }

    #[tokio::test]
    async fn hold_creation_is_atomic_against_available_stock() {
        let state = state().await;
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('RARE-1','Rare part',3,0,0)").execute(&state.pool).await.unwrap();
        let first = HoldInput {
            inventory_id: 1,
            quantity: 2,
            customer: "North shop".into(),
            order_note: None,
            operator_name: "Asha".into(),
            duration_minutes: 30,
        };
        let _ = create_hold(State(state.clone()), Json(first))
            .await
            .unwrap();
        let second = HoldInput {
            inventory_id: 1,
            quantity: 2,
            customer: "South shop".into(),
            order_note: None,
            operator_name: "Ben".into(),
            duration_minutes: 30,
        };
        let error = create_hold(State(state.clone()), Json(second))
            .await
            .unwrap_err();
        assert!(matches!(error, ApiError::Conflict(_)));
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM holds WHERE status='active'")
            .fetch_one(&state.pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn conversion_reduces_stock_and_audits_outcome() {
        let state = state().await;
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('ONE','Part',5,0,0)").execute(&state.pool).await.unwrap();
        let hold = HoldInput {
            inventory_id: 1,
            quantity: 3,
            customer: "Buyer".into(),
            order_note: None,
            operator_name: "Lee".into(),
            duration_minutes: 30,
        };
        let (_, Json(created)) = create_hold(State(state.clone()), Json(hold)).await.unwrap();
        let id = created["id"].as_str().unwrap();
        let mut conn = state.pool.acquire().await.unwrap();
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *conn)
            .await
            .unwrap();
        resolve_hold_locked(&mut conn, id, "convert", "Supervisor", db::now())
            .await
            .unwrap();
        sqlx::query("COMMIT").execute(&mut *conn).await.unwrap();
        drop(conn);
        let stock: i64 = sqlx::query_scalar("SELECT on_hand FROM inventory WHERE id=1")
            .fetch_one(&state.pool)
            .await
            .unwrap();
        assert_eq!(stock, 2);
        let event: String =
            sqlx::query_scalar("SELECT event FROM audit_log ORDER BY id DESC LIMIT 1")
                .fetch_one(&state.pool)
                .await
                .unwrap();
        assert_eq!(event, "hold.converted");
    }

    #[test]
    fn validation_rejects_ambiguous_skus_and_short_pins() {
        assert!(normalize_sku("bad sku!").is_err());
        assert!(validate_pin("1234").is_err());
        assert_eq!(normalize_sku(" ab-12 ").unwrap(), "AB-12");
    }
}
