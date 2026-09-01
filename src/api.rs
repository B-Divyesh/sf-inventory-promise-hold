use argon2::{
    password_hash::{
        rand_core::OsRng as PasswordOsRng, PasswordHash, PasswordHasher, PasswordVerifier,
        SaltString,
    },
    Argon2,
};
use axum::{
    body::Body,
    extract::{ConnectInfo, Path, Request, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uuid::Uuid;

use crate::{
    auth::{AuthError, AuthMode, AuthService, Principal, Role},
    db,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub build_sha: String,
    auth: AuthService,
    login_guard: Arc<LoginGuard>,
    api_guard: Arc<ApiGuard>,
}

impl AppState {
    pub fn new(pool: SqlitePool, build_sha: String) -> Self {
        Self::with_auth(pool, build_sha, AuthService::local_for_tests())
    }

    pub fn with_auth(pool: SqlitePool, build_sha: String, auth: AuthService) -> Self {
        Self {
            pool,
            build_sha,
            auth,
            login_guard: Arc::new(LoginGuard::new()),
            api_guard: Arc::new(ApiGuard::new()),
        }
    }
}

struct ApiGuard {
    clients: Mutex<HashMap<String, VecDeque<Instant>>>,
}

impl ApiGuard {
    fn new() -> Self {
        Self {
            clients: Mutex::new(HashMap::new()),
        }
    }

    fn begin(&self, client: &str, write: bool) -> Result<(), ApiError> {
        let (limit, window) = if write {
            (20usize, Duration::from_secs(60))
        } else {
            (80usize, Duration::from_secs(60))
        };
        let now = Instant::now();
        let bucket = format!("{}:{}", if write { "write" } else { "read" }, client);
        let mut clients = self.clients.lock().unwrap_or_else(|lock| lock.into_inner());
        let requests = clients.entry(bucket).or_default();
        requests.retain(|time| now.duration_since(*time) < window);
        if requests.len() >= limit {
            let retry_after = requests
                .front()
                .map(|first| {
                    window
                        .saturating_sub(now.duration_since(*first))
                        .as_secs()
                        .max(1)
                })
                .unwrap_or(1);
            return Err(ApiError::RateLimited(retry_after));
        }
        requests.push_back(now);
        Ok(())
    }
}

struct LoginGuard {
    attempts: Mutex<LoginAttempts>,
    concurrent: Arc<Semaphore>,
}

#[derive(Default)]
struct LoginAttempts {
    global: VecDeque<Instant>,
    clients: HashMap<String, VecDeque<Instant>>,
}

impl LoginGuard {
    fn new() -> Self {
        Self {
            attempts: Mutex::new(LoginAttempts::default()),
            concurrent: Arc::new(Semaphore::new(4)),
        }
    }

    fn begin(&self, client: &str) -> Result<OwnedSemaphorePermit, ApiError> {
        let permit = self
            .concurrent
            .clone()
            .try_acquire_owned()
            .map_err(|_| ApiError::RateLimited(60))?;
        let now = Instant::now();
        let window = Duration::from_secs(60);
        let mut attempts = self
            .attempts
            .lock()
            .unwrap_or_else(|lock| lock.into_inner());
        attempts
            .global
            .retain(|time| now.duration_since(*time) < window);
        if attempts.global.len() >= 30 {
            return Err(ApiError::RateLimited(60));
        }
        let client_attempts = attempts.clients.entry(client.to_owned()).or_default();
        client_attempts.retain(|time| now.duration_since(*time) < window);
        if client_attempts.len() >= 10 {
            return Err(ApiError::RateLimited(60));
        }
        client_attempts.push_back(now);
        attempts.global.push_back(now);
        Ok(permit)
    }
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
    #[error("Too many requests. Wait briefly and try again.")]
    RateLimited(u64),
    #[error("The server could not complete that action. Try again.")]
    Internal(#[from] sqlx::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let mut response = (status, Json(json!({ "error": self.to_string() }))).into_response();
        if let Self::RateLimited(retry_after) = self {
            response.headers_mut().insert(
                header::RETRY_AFTER,
                HeaderValue::from_str(&retry_after.to_string())
                    .unwrap_or(HeaderValue::from_static("1")),
            );
        }
        if status == StatusCode::UNAUTHORIZED {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        }
        response
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/status", get(status))
        .route("/bootstrap", get(bootstrap))
        .route("/setup", post(setup))
        .route("/session", post(login).delete(logout))
        .route("/inventory", post(create_inventory))
        .route("/inventory/{id}", post(update_inventory))
        .route("/holds", post(create_hold))
        .route("/holds/{id}/resolve", post(resolve_hold))
        .route("/audit", get(audit))
        .route("/export.csv", get(export_csv))
        .route("/data-retention", get(get_retention).post(set_retention))
        .route("/location", axum::routing::delete(delete_location))
        .route("/auth/config", get(auth_config))
}

pub async fn rate_limit(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let is_write = !matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    );
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|connect| connect.0)
        .unwrap_or_else(|| "127.0.0.1:0".parse().expect("valid loopback address"));
    let client = client_identity(peer, request.headers());
    match state.api_guard.begin(&client, is_write) {
        Ok(()) => next.run(request).await,
        Err(error) => error.into_response(),
    }
}

#[derive(Serialize)]
struct AuthConfig {
    mode: &'static str,
}

async fn auth_config(State(state): State<AppState>) -> Json<AuthConfig> {
    Json(AuthConfig {
        mode: state.auth.mode.as_str(),
    })
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
    role: String,
}

#[derive(Serialize)]
struct SetupStatus {
    setup_required: bool,
    server_time: i64,
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

async fn status(State(state): State<AppState>) -> Result<Json<SetupStatus>, ApiError> {
    let setting = sqlx::query("SELECT location_name FROM settings WHERE singleton = 1")
        .fetch_optional(&state.pool)
        .await?;
    Ok(Json(SetupStatus {
        setup_required: setting.is_none(),
        server_time: db::now(),
    }))
}

async fn bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Bootstrap>, ApiError> {
    let principal = require_role(&state, &headers, Role::Staff).await?;
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
        role: principal.role.as_str().into(),
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
    pin: Option<String>,
}

#[derive(Serialize)]
struct SessionOutput {
    token: String,
    expires_at: i64,
    role: String,
}

async fn setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<SetupInput>,
) -> Result<Json<SessionOutput>, ApiError> {
    let location = required_text(&input.location_name, "Location name", 80)?;
    let (hash, setup_principal) = match state.auth.mode {
        AuthMode::Local => {
            let pin = input
                .pin
                .ok_or_else(|| ApiError::BadRequest("Supervisor PIN is required.".into()))?;
            validate_pin(&pin)?;
            let hash = tokio::task::spawn_blocking(move || {
                let salt = SaltString::generate(&mut PasswordOsRng);
                Argon2::default()
                    .hash_password(pin.as_bytes(), &salt)
                    .map(|value| value.to_string())
            })
            .await
            .map_err(|_| ApiError::BadRequest("Could not secure that PIN. Try again.".into()))?
            .map_err(|_| ApiError::BadRequest("Could not secure that PIN. Try again.".into()))?;
            (
                hash,
                Principal {
                    oid: "local-supervisor".into(),
                    role: Role::Supervisor,
                },
            )
        }
        AuthMode::Ciam => (
            "ciam-managed".into(),
            require_role(&state, &headers, Role::Supervisor).await?,
        ),
    };
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
    sqlx::query("INSERT INTO audit_log(event, entity_type, entity_id, actor, details_json, created_at) VALUES('location.setup', 'location', '1', ?, ?, ?)")
        .bind(&setup_principal.oid)
        .bind(json!({ "location_name": location }).to_string())
        .bind(now)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    let session = if matches!(state.auth.mode, AuthMode::Local) {
        create_session(&state.pool, Role::Supervisor).await?
    } else {
        SessionOutput {
            token: String::new(),
            expires_at: 0,
            role: Role::Supervisor.as_str().into(),
        }
    };
    Ok(Json(session))
}

#[derive(Deserialize)]
struct LoginInput {
    pin: String,
}

async fn login(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(input): Json<LoginInput>,
) -> Result<Json<SessionOutput>, ApiError> {
    if matches!(state.auth.mode, AuthMode::Ciam) {
        return Err(ApiError::Unauthorized(
            "Use Sociobot sign-in to open the live promise desk.".into(),
        ));
    }
    let client = client_identity(peer, &headers);
    let _attempt = state.login_guard.begin(&client)?;
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
    Ok(Json(create_session(&state.pool, Role::Supervisor).await?))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Result<StatusCode, ApiError> {
    if matches!(state.auth.mode, AuthMode::Ciam) {
        return Ok(StatusCode::NO_CONTENT);
    }
    let token = bearer(&headers)?;
    sqlx::query("DELETE FROM sessions WHERE token_hash = ?")
        .bind(token_hash(token))
        .execute(&state.pool)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_session(pool: &SqlitePool, role: Role) -> Result<SessionOutput, ApiError> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    let token = hex(&bytes);
    let now = db::now();
    let expires_at = now + 12 * 60 * 60;
    sqlx::query("DELETE FROM sessions WHERE expires_at <= ?")
        .bind(now)
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT INTO sessions(token_hash, role, expires_at, created_at) VALUES(?, ?, ?, ?)",
    )
    .bind(token_hash(&token))
    .bind(role.as_str())
    .bind(expires_at)
    .bind(now)
    .execute(pool)
    .await?;
    Ok(SessionOutput {
        token,
        expires_at,
        role: role.as_str().into(),
    })
}

async fn require_role(
    state: &AppState,
    headers: &HeaderMap,
    required: Role,
) -> Result<Principal, ApiError> {
    let principal = match state.auth.mode {
        AuthMode::Local => {
            let token = bearer(headers)?;
            let role: Option<String> = sqlx::query_scalar(
                "SELECT role FROM sessions WHERE token_hash = ? AND expires_at > ?",
            )
            .bind(token_hash(token))
            .bind(db::now())
            .fetch_optional(&state.pool)
            .await?;
            let role = match role.as_deref() {
                Some("supervisor") => Role::Supervisor,
                Some("staff") => Role::Staff,
                _ => {
                    return Err(ApiError::Unauthorized(
                        "Local access has expired. Sign in again.".into(),
                    ))
                }
            };
            Principal {
                oid: "local-user".into(),
                role,
            }
        }
        AuthMode::Ciam => state
            .auth
            .validate(bearer(headers).ok())
            .await
            .map_err(auth_error)?,
    };
    if principal.role.allows(required) {
        Ok(principal)
    } else {
        Err(ApiError::Unauthorized(
            "A supervisor must complete that action.".into(),
        ))
    }
}

async fn require_supervisor(state: &AppState, headers: &HeaderMap) -> Result<Principal, ApiError> {
    require_role(state, headers, Role::Supervisor).await
}

fn auth_error(error: AuthError) -> ApiError {
    ApiError::Unauthorized(error.to_string())
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
    require_supervisor(&state, &headers).await?;
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
    require_supervisor(&state, &headers).await?;
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

struct HoldText {
    customer: String,
    operator: String,
    note: String,
}

async fn create_hold(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<HoldInput>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let principal = require_role(&state, &headers, Role::Staff).await?;
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
    let text = HoldText {
        customer: required_text(&input.customer, "Customer", 120)?,
        operator: required_text(&input.operator_name, "Operator name", 80)?,
        note: limited_text(input.order_note.as_deref().unwrap_or(""), "Order note", 300)?,
    };
    let now = db::now();
    let expires_at = now + input.duration_minutes * 60;
    let id = Uuid::new_v4().to_string();
    let mut conn = state.pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    let outcome = create_hold_locked(
        &mut conn,
        &input,
        &id,
        &text,
        &principal.oid,
        now,
        expires_at,
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
    text: &HoldText,
    audit_actor: &str,
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
        .bind(id).bind(input.inventory_id).bind(input.quantity).bind(&text.customer).bind(&text.note).bind(&text.operator).bind(now).bind(expires_at)
        .execute(&mut **conn).await?;
    let details = json!({"sku": item.get::<String, _>("sku"), "item_name": item.get::<String, _>("name"), "quantity": input.quantity, "expires_at": expires_at});
    sqlx::query("INSERT INTO audit_log(event, entity_type, entity_id, actor, details_json, created_at) VALUES('hold.created', 'hold', ?, ?, ?, ?)")
        .bind(id).bind(audit_actor).bind(details.to_string()).bind(now).execute(&mut **conn).await?;
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
    let principal = require_supervisor(&state, &headers).await?;
    let _actor = required_text(&input.actor, "Supervisor name", 80)?;
    if input.action != "convert" && input.action != "release" {
        return Err(ApiError::BadRequest(
            "Action must be convert or release.".into(),
        ));
    }
    let now = db::now();
    let mut conn = state.pool.acquire().await?;
    sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
    let outcome = resolve_hold_locked(&mut conn, &id, &input.action, &principal.oid, now).await;
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
    require_supervisor(&state, &headers).await?;
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
    require_supervisor(&state, &headers).await?;
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

#[derive(Serialize)]
struct RetentionOutput {
    retention_days: i64,
}

#[derive(Deserialize)]
struct RetentionInput {
    retention_days: i64,
}

async fn get_retention(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<RetentionOutput>, ApiError> {
    require_supervisor(&state, &headers).await?;
    let retention_days =
        sqlx::query_scalar::<_, i64>("SELECT retention_days FROM settings WHERE singleton = 1")
            .fetch_optional(&state.pool)
            .await?
            .unwrap_or(90);
    Ok(Json(RetentionOutput { retention_days }))
}

async fn set_retention(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<RetentionInput>,
) -> Result<Json<RetentionOutput>, ApiError> {
    let principal = require_supervisor(&state, &headers).await?;
    if !(30..=730).contains(&input.retention_days) {
        return Err(ApiError::BadRequest(
            "Retention must be between 30 and 730 days.".into(),
        ));
    }
    sqlx::query("UPDATE settings SET retention_days = ? WHERE singleton = 1")
        .bind(input.retention_days)
        .execute(&state.pool)
        .await?;
    db::redact_retained_hold_details(&state.pool).await?;
    audit_event(
        &state.pool,
        "privacy.retention_changed",
        "location",
        "1",
        &principal.oid,
        json!({"retention_days": input.retention_days}),
    )
    .await?;
    Ok(Json(RetentionOutput {
        retention_days: input.retention_days,
    }))
}

#[derive(Deserialize)]
struct DeleteLocationInput {
    confirmation: String,
}

async fn delete_location(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(input): Json<DeleteLocationInput>,
) -> Result<StatusCode, ApiError> {
    require_supervisor(&state, &headers).await?;
    if input.confirmation != "DELETE" {
        return Err(ApiError::BadRequest(
            "Type DELETE to permanently erase this location's data.".into(),
        ));
    }
    // A full location erasure is the explicit, documented exception to the
    // append-only ledger. It removes all operational and audit data together.
    sqlx::raw_sql(
        "DROP TRIGGER IF EXISTS audit_log_no_update; DROP TRIGGER IF EXISTS audit_log_no_delete;",
    )
    .execute(&state.pool)
    .await?;
    sqlx::raw_sql("DELETE FROM holds; DELETE FROM inventory; DELETE FROM sessions; DELETE FROM audit_log; DELETE FROM settings;")
        .execute(&state.pool).await?;
    sqlx::raw_sql("CREATE TRIGGER audit_log_no_update BEFORE UPDATE ON audit_log BEGIN SELECT RAISE(ABORT, 'audit log is append-only'); END; CREATE TRIGGER audit_log_no_delete BEFORE DELETE ON audit_log BEGIN SELECT RAISE(ABORT, 'audit log is append-only'); END;")
        .execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
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

fn client_identity(peer: SocketAddr, headers: &HeaderMap) -> String {
    // The ingress forwards the public client address as the first XFF hop.
    // Prefer it over the socket peer or proxy-specific headers.
    for name in ["x-forwarded-for", "x-envoy-external-address"] {
        if let Some(ip) = headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .and_then(|value| value.parse::<std::net::IpAddr>().ok())
        {
            return ip.to_string();
        }
    }
    peer.ip().to_string()
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
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::time::Duration;
    use tower::ServiceExt;

    async fn state() -> AppState {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::db::migrate(&pool).await.unwrap();
        AppState::new(pool, "test".into())
    }

    async fn auth_headers(state: &AppState) -> HeaderMap {
        role_headers(state, Role::Supervisor).await
    }

    async fn role_headers(state: &AppState, role: Role) -> HeaderMap {
        let session = create_session(&state.pool, role).await.unwrap();
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", session.token)).unwrap(),
        );
        headers
    }

    // @claim:contested-stock-protection
    #[tokio::test]
    async fn claim_contested_stock_protection_allows_only_one_competing_hold() {
        let directory = tempfile::tempdir().unwrap();
        let options = SqliteConnectOptions::new()
            .filename(directory.path().join("race.db"))
            .create_if_missing(true)
            .busy_timeout(Duration::from_secs(5));
        let pool = SqlitePoolOptions::new()
            .max_connections(4)
            .connect_with(options)
            .await
            .unwrap();
        crate::db::migrate(&pool).await.unwrap();
        let state = AppState::new(pool, "test".into());
        let headers = auth_headers(&state).await;
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('RARE-1','Rare part',3,0,0)").execute(&state.pool).await.unwrap();
        let barrier = std::sync::Arc::new(tokio::sync::Barrier::new(3));
        let attempt = |customer: &'static str, operator: &'static str| {
            let state = state.clone();
            let barrier = barrier.clone();
            let headers = headers.clone();
            tokio::spawn(async move {
                barrier.wait().await;
                create_hold(
                    State(state),
                    headers,
                    Json(HoldInput {
                        inventory_id: 1,
                        quantity: 2,
                        customer: customer.into(),
                        order_note: None,
                        operator_name: operator.into(),
                        duration_minutes: 30,
                    }),
                )
                .await
            })
        };
        let first = attempt("North shop", "Asha");
        let second = attempt("South shop", "Ben");
        barrier.wait().await;
        let (first, second) = tokio::join!(first, second);
        let results = [first.unwrap(), second.unwrap()];
        assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
        assert_eq!(
            results
                .iter()
                .filter(|result| matches!(result, Err(ApiError::Conflict(_))))
                .count(),
            1
        );
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM holds WHERE status='active'")
            .fetch_one(&state.pool)
            .await
            .unwrap();
        assert_eq!(count, 1);
    }

    // @claim:append-only-audit
    #[tokio::test]
    async fn claim_append_only_audit_keeps_hold_outcomes_immutable() {
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
        let headers = auth_headers(&state).await;
        let (_, Json(created)) = create_hold(State(state.clone()), headers, Json(hold))
            .await
            .unwrap();
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
        let created_details: String = sqlx::query_scalar(
            "SELECT details_json FROM audit_log WHERE event = 'hold.created' ORDER BY id ASC LIMIT 1",
        )
        .fetch_one(&state.pool)
        .await
        .unwrap();
        assert!(!created_details.contains("Buyer"));
        assert!(!created_details.contains("Lee"));
        let deletion = sqlx::query("DELETE FROM audit_log")
            .execute(&state.pool)
            .await;
        assert!(deletion.is_err(), "audit rows must be immutable");
    }

    #[test]
    fn validation_rejects_ambiguous_skus_and_short_pins() {
        assert!(normalize_sku("bad sku!").is_err());
        assert!(validate_pin("1234").is_err());
        assert_eq!(normalize_sku(" ab-12 ").unwrap(), "AB-12");
    }

    #[test]
    fn login_client_identity_uses_the_validated_proxy_address() {
        let peer = "10.0.0.2:1234".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "203.0.113.8, 10.0.0.2".parse().unwrap());
        assert_eq!(client_identity(peer, &headers), "203.0.113.8");
        headers.insert("x-envoy-external-address", "198.51.100.4".parse().unwrap());
        assert_eq!(client_identity(peer, &headers), "203.0.113.8");
    }

    #[test]
    fn login_guard_limits_each_client_and_concurrent_hashes() {
        let guard = LoginGuard::new();
        for _ in 0..10 {
            drop(guard.begin("192.0.2.10").unwrap());
        }
        assert!(matches!(
            guard.begin("192.0.2.10"),
            Err(ApiError::RateLimited(_))
        ));

        let guard = LoginGuard::new();
        for index in 0..30 {
            drop(guard.begin(&format!("198.51.100.{index}")));
        }
        assert!(matches!(
            guard.begin("203.0.113.1"),
            Err(ApiError::RateLimited(_))
        ));

        let guard = LoginGuard::new();
        let permits: Vec<_> = (0..4)
            .map(|index| guard.begin(&format!("192.0.2.{index}")))
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(matches!(
            guard.begin("192.0.2.99"),
            Err(ApiError::RateLimited(_))
        ));
        drop(permits);
    }

    #[tokio::test]
    async fn operational_data_and_holds_require_a_session() {
        let state = state().await;
        assert!(status(State(state.clone())).await.unwrap().0.setup_required);

        sqlx::query("INSERT INTO settings(singleton,location_name,supervisor_pin_hash,created_at) VALUES(1,'Private stockroom','hash',0)")
            .execute(&state.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('PRIVATE-1','Customer item',2,0,0)")
            .execute(&state.pool)
            .await
            .unwrap();
        assert!(!status(State(state.clone())).await.unwrap().0.setup_required);

        assert!(matches!(
            bootstrap(State(state.clone()), HeaderMap::new()).await,
            Err(ApiError::Unauthorized(_))
        ));
        let hold = HoldInput {
            inventory_id: 1,
            quantity: 1,
            customer: "Private customer".into(),
            order_note: None,
            operator_name: "Operator".into(),
            duration_minutes: 30,
        };
        assert!(matches!(
            create_hold(State(state.clone()), HeaderMap::new(), Json(hold)).await,
            Err(ApiError::Unauthorized(_))
        ));

        let authenticated = bootstrap(State(state.clone()), auth_headers(&state).await)
            .await
            .unwrap();
        assert_eq!(authenticated.0.inventory.len(), 1);
    }

    // @claim:role-boundary
    #[tokio::test]
    async fn claim_role_boundary_staff_can_hold_but_not_change_or_resolve_stock() {
        let state = state().await;
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('ROLE-1','Role-bound part',4,0,0)")
            .execute(&state.pool).await.unwrap();
        let staff = role_headers(&state, Role::Staff).await;
        let (_, Json(created)) = create_hold(
            State(state.clone()),
            staff.clone(),
            Json(HoldInput {
                inventory_id: 1,
                quantity: 1,
                customer: "Staff customer".into(),
                order_note: None,
                operator_name: "Staff member".into(),
                duration_minutes: 30,
            }),
        )
        .await
        .unwrap();
        let changed = update_inventory(
            State(state.clone()),
            Path(1),
            staff.clone(),
            Json(InventoryInput {
                sku: "ROLE-1".into(),
                name: "Role-bound part".into(),
                on_hand: 5,
            }),
        )
        .await;
        assert!(matches!(changed, Err(ApiError::Unauthorized(_))));
        let resolved = resolve_hold(
            State(state),
            Path(created["id"].as_str().unwrap().to_string()),
            staff,
            Json(ResolveInput {
                action: "release".into(),
                actor: "Staff member".into(),
            }),
        )
        .await;
        assert!(matches!(resolved, Err(ApiError::Unauthorized(_))));
    }

    // @claim:rate-limit
    #[tokio::test]
    async fn claim_rate_limit_returns_retry_after_for_excessive_status_requests() {
        let state = state().await;
        let app = crate::build_app(state, tempfile::tempdir().unwrap().keep());
        for _ in 0..80 {
            let response = app
                .clone()
                .oneshot(
                    axum::http::Request::builder()
                        .uri("/api/status")
                        .header("x-forwarded-for", "203.0.113.21")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/status")
                    .header("x-forwarded-for", "203.0.113.21")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert!(response.headers().get(header::RETRY_AFTER).is_some());
    }

    // @claim:automatic-expiry
    #[tokio::test]
    async fn claim_automatic_expiry_releases_stock_and_records_an_outcome() {
        let state = state().await;
        let now = db::now();
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('DUE-1','Due part',5,?,?)")
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO holds(id,inventory_id,quantity,customer,order_note,operator_name,status,created_at,expires_at) VALUES('due-hold',1,2,'Due customer','','Mina','active',?,?)")
            .bind(now - 600)
            .bind(now - 1)
            .execute(&state.pool)
            .await
            .unwrap();

        assert_eq!(db::expire_due(&state.pool).await.unwrap(), 1);
        let Json(bootstrapped) = bootstrap(State(state.clone()), auth_headers(&state).await)
            .await
            .unwrap();
        assert_eq!(bootstrapped.inventory[0].available, 5);
        assert!(bootstrapped.active_holds.is_empty());
        assert_eq!(bootstrapped.recent_outcomes[0].status, "expired");
        assert_eq!(
            bootstrapped.recent_outcomes[0].resolved_by.as_deref(),
            Some("Clock")
        );
        assert_eq!(
            sqlx::query_scalar::<_, String>(
                "SELECT event FROM audit_log WHERE entity_id='due-hold'"
            )
            .fetch_one(&state.pool)
            .await
            .unwrap(),
            "hold.expired"
        );
    }

    // @claim:location-erasure
    #[tokio::test]
    async fn claim_location_erasure_removes_operational_data_and_restores_audit_protection() {
        let state = state().await;
        let now = db::now();
        sqlx::query("INSERT INTO settings(singleton,location_name,supervisor_pin_hash,created_at) VALUES(1,'Erase test','hash',?)")
            .bind(now)
            .execute(&state.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO inventory(sku,name,on_hand,created_at,updated_at) VALUES('ERASE-1','Erase part',3,?,?)")
            .bind(now)
            .bind(now)
            .execute(&state.pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO holds(id,inventory_id,quantity,customer,order_note,operator_name,status,created_at,expires_at) VALUES('erase-hold',1,1,'Customer','','Mina','active',?,?)")
            .bind(now)
            .bind(now + 300)
            .execute(&state.pool)
            .await
            .unwrap();
        audit_event(
            &state.pool,
            "hold.created",
            "hold",
            "erase-hold",
            "Mina",
            json!({"quantity": 1}),
        )
        .await
        .unwrap();

        let response = delete_location(
            State(state.clone()),
            auth_headers(&state).await,
            Json(DeleteLocationInput {
                confirmation: "DELETE".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(response, StatusCode::NO_CONTENT);
        for table in ["settings", "inventory", "holds", "sessions", "audit_log"] {
            let query = format!("SELECT COUNT(*) FROM {table}");
            assert_eq!(
                sqlx::query_scalar::<_, i64>(&query)
                    .fetch_one(&state.pool)
                    .await
                    .unwrap(),
                0,
                "{table} must be erased"
            );
        }
        assert!(status(State(state.clone())).await.unwrap().0.setup_required);

        audit_event(&state.pool, "test.event", "test", "1", "Tester", json!({}))
            .await
            .unwrap();
        assert!(sqlx::query("DELETE FROM audit_log")
            .execute(&state.pool)
            .await
            .is_err());
    }
}
