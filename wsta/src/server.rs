// trade/wsta/src/server.rs
//
// axum host for final browser/Makepad page.
// Browser controls are not sent over HTTP.
// HTTP only serves the page/assets and lightweight state-free documents.

use crate::actor::ActorHandle;
use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Request},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use tower_http::{cors::CorsLayer, services::ServeDir};

#[derive(Debug, Clone)]
pub struct WstaServerConfig {
    pub http_bind: String,
    pub seek_backend_addr: String,
}

impl WstaServerConfig {
    pub fn from_env() -> Self {
        Self {
            http_bind: std::env::var("WSTA_HTTP_BIND")
                .unwrap_or_else(|_| "0.0.0.0:8088".to_string()),

            // Existing dr_seek server binds QUIC on 0.0.0.0:4433.
            seek_backend_addr: std::env::var("WSTA_SEEK_BACKEND_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:4433".to_string()),
        }
    }
}

#[derive(Clone)]
struct AppState {
    _actor: ActorHandle,
}

pub async fn run(
    cfg: WstaServerConfig,
    actor: ActorHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { _actor: actor };

    let app = Router::new()
        .route("/", get(index))
        .nest_service(
            "/makepad",
            ServeDir::new("wsta_makepad/target/makepad-wasm")
                .fallback(ServeDir::new("wsta_makepad/resources/web")),
        )
        .nest_service("/resources", ServeDir::new("wsta_makepad/resources"))
        .nest_service("/makepad_resources", ServeDir::new("wsta_makepad/resources"))
        .nest_service("/assets/images", ServeDir::new("wsta/assets/images"))
        .route("/status", get(status))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(add_cross_origin_isolation_headers))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.http_bind).await?;
    log::info!("WSTA serving browser shell at http://{}/", cfg.http_bind);

    axum::serve(listener, app).await?;

    Ok(())
}


async fn add_cross_origin_isolation_headers(
    request: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Makepad WASM packages use shared-memory/thread-oriented browser features.
    // Browsers require cross-origin isolation for those paths. Apply these to
    // the shell and static assets so the UI can render instead of hanging at
    // the generated Makepad "loading" state.
    headers.insert(
        "Cross-Origin-Opener-Policy",
        HeaderValue::from_static("same-origin"),
    );
    headers.insert(
        "Cross-Origin-Embedder-Policy",
        HeaderValue::from_static("require-corp"),
    );
    headers.insert(
        "Cross-Origin-Resource-Policy",
        HeaderValue::from_static("same-origin"),
    );

    // Be explicit for wasm/js assets when tower's extension mapping is absent
    // or conservative on embedded systems.
    if let Some(path) = response.extensions().get::<String>() {
        let _ = path;
    }

    response
}

async fn index(State(_state): State<AppState>) -> Html<&'static str> {
    Html(crate::web::INDEX_HTML)
}

async fn status() -> impl IntoResponse {
    (
        [("content-type", "application/json; charset=utf-8")],
        r#"{"ok":true,"service":"wsta","role":"browser shell host only; controls use WebTransport"}"#,
    )
}
