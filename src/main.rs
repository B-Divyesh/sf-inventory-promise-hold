mod api;
mod db;

use std::{env, net::SocketAddr, path::PathBuf};

use axum::{
    http::{header, HeaderName, HeaderValue},
    routing::get,
    Router,
};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::info;
use uuid::Uuid;

pub use api::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8080);
    let database_path =
        env::var("DATABASE_PATH").unwrap_or_else(|_| "/data/stock-promise.db".into());
    if let Some(parent) = std::path::Path::new(&database_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let options = SqliteConnectOptions::new()
        .filename(&database_path)
        .create_if_missing(true)
        .foreign_keys(true)
        .journal_mode(SqliteJournalMode::Wal);
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    db::migrate(&pool).await?;
    let instance_status = db::ensure_instance_id(&pool, &Uuid::new_v4().to_string()).await?;

    let state = AppState {
        pool,
        build_sha: env::var("BUILD_SHA").unwrap_or_else(|_| "development".into()),
    };
    let frontend =
        PathBuf::from(env::var("FRONTEND_DIR").unwrap_or_else(|_| "frontend/dist".into()));
    let app = build_app(state, frontend);
    let address = SocketAddr::from(([0, 0, 0, 0], port));

    info!(
        port,
        database = %database_path,
        instance_identity = %instance_status,
        "configuration ready (PORT defaults to 8080; database and instance identity persist locally)"
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

pub fn build_app(state: AppState, frontend: PathBuf) -> Router {
    let fallback =
        ServeDir::new(&frontend).not_found_service(ServeFile::new(frontend.join("index.html")));
    Router::new()
        .route("/health", get(api::health))
        .nest("/api", api::routes())
        .fallback_service(fallback)
        .layer(TraceLayer::new_for_http())
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; connect-src 'self' https://api.sociobot.in https://pilot-api.sociobot.in; frame-ancestors 'none'; base-uri 'self'; form-action 'self' https://api.sociobot.in https://pilot-api.sociobot.in"),
        ))
        .with_state(state)
}

async fn shutdown_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("ctrl-c handler") };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("termination handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
