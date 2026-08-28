mod api;
mod db;

use std::{env, net::SocketAddr, path::PathBuf};

use axum::{
    body::Body,
    extract::Request,
    http::{header, HeaderName, HeaderValue},
    middleware::{self, Next},
    response::Response,
    routing::{get, get_service},
    Router,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing::info;
use uuid::Uuid;

pub use api::AppState;

const COMPILED_BUILD_SHA: &str = match option_env!("BUILD_SHA") {
    Some(value) => value,
    None => "dev",
};

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
        .busy_timeout(std::time::Duration::from_secs(5));
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(options)
        .await?;
    let schema_status = db::prepare_schema(&pool).await?;
    let instance_status = db::ensure_instance_id(&pool, &Uuid::new_v4().to_string()).await?;

    let state = AppState::new(
        pool,
        env::var("BUILD_SHA").unwrap_or_else(|_| COMPILED_BUILD_SHA.into()),
    );
    let expiry_pool = state.pool.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(error) = db::expire_due(&expiry_pool).await {
                tracing::warn!(%error, "automatic expiry sweep failed");
            }
        }
    });
    let frontend = PathBuf::from(env::var("FRONTEND_DIR").unwrap_or_else(|_| "dist".into()));
    let app = build_app(state, frontend);
    let address = SocketAddr::from(([0, 0, 0, 0], port));

    info!(
        port,
        database = %database_path,
        schema = %schema_status,
        instance_identity = %instance_status,
        "configuration ready (PORT defaults to 8080; database and instance identity persist locally)"
    );
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;
    Ok(())
}

pub fn build_app(state: AppState, frontend: PathBuf) -> Router {
    let index = frontend.join("index.html");
    let fallback = ServeDir::new(&frontend).not_found_service(ServeFile::new(index.clone()));
    Router::new()
        .route("/health", get(api::health))
        .nest("/api", api::routes())
        .route_service("/privacy", get_service(ServeFile::new(index.clone())))
        .route_service("/terms", get_service(ServeFile::new(index)))
        .fallback_service(fallback)
        .layer(middleware::from_fn(response_policy))
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
            HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; connect-src 'self' https://api.sociobot.in https://pilot-api.sociobot.in; frame-ancestors 'none'; base-uri 'self'; form-action 'self' https://api.sociobot.in https://pilot-api.sociobot.in"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static(
                "camera=(), geolocation=(), microphone=(), payment=(), usb=()",
            ),
        ))
        .with_state(state)
}

async fn response_policy(request: Request, next: Next) -> Response<Body> {
    let path = request.uri().path().to_owned();
    let mut response = next.run(request).await;
    let policy = if path.starts_with("/api/") || path == "/health" {
        "no-store"
    } else if path == "/sw.js" {
        "no-cache, no-store, must-revalidate"
    } else if path.starts_with("/assets/index-") {
        "public, max-age=31536000, immutable"
    } else if path == "/"
        || path
            .rsplit('/')
            .next()
            .is_some_and(|segment| !segment.contains('.'))
    {
        "no-cache, must-revalidate"
    } else {
        "public, max-age=86400"
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(policy));
    response
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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{Method, Request, StatusCode};
    use http_body_util::BodyExt;
    use sqlx::sqlite::SqlitePoolOptions;
    use tower::ServiceExt;

    async fn test_app() -> Router {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        db::migrate(&pool).await.unwrap();
        let directory = tempfile::tempdir().unwrap();
        std::fs::write(
            directory.path().join("index.html"),
            "<!doctype html><title>test</title>",
        )
        .unwrap();
        std::fs::write(directory.path().join("sw.js"), "// worker").unwrap();
        std::fs::create_dir(directory.path().join("assets")).unwrap();
        std::fs::write(directory.path().join("assets/index-test.js"), "export {};").unwrap();
        let path = directory.keep();
        build_app(AppState::new(pool, "exact-build-sha".into()), path)
    }

    async fn response(app: &Router, method: Method, uri: &str) -> Response<Body> {
        app.clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn legal_routes_and_response_policies_are_explicit() {
        let app = test_app().await;
        let privacy = response(&app, Method::GET, "/privacy").await;
        assert_eq!(privacy.status(), StatusCode::OK);
        assert_eq!(
            privacy.headers()[header::CACHE_CONTROL],
            "no-cache, must-revalidate"
        );
        assert!(privacy.headers().contains_key("strict-transport-security"));
        assert!(privacy.headers().contains_key("permissions-policy"));

        let terms = response(&app, Method::HEAD, "/terms").await;
        assert_eq!(terms.status(), StatusCode::OK);
        let asset = response(&app, Method::GET, "/assets/index-test.js").await;
        assert_eq!(
            asset.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        let worker = response(&app, Method::GET, "/sw.js").await;
        assert_eq!(
            worker.headers()[header::CACHE_CONTROL],
            "no-cache, no-store, must-revalidate"
        );
        let api = response(&app, Method::GET, "/api/bootstrap").await;
        assert_eq!(api.headers()[header::CACHE_CONTROL], "no-store");
    }

    #[tokio::test]
    async fn health_reports_the_exact_build_identity() {
        let app = test_app().await;
        let response = response(&app, Method::GET, "/health").await;
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(payload["build_sha"], "exact-build-sha");
    }
}
