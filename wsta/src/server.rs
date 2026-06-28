// trade/wsta/src/server.rs
//
// axum host for final browser/Makepad page.
// Browser controls are not sent over HTTP.
// HTTP only serves the page/assets and lightweight state-free documents.

use crate::actor::ActorHandle;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
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
    actor: ActorHandle,
}

pub async fn run(
    cfg: WstaServerConfig,
    actor: ActorHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState { actor };

    let app = Router::new()
        .route("/", get(index))
        .route("/makepad/", get(index))
        .route("/assets/wsta.css", get(css))
        .route("/assets/wsta.js", get(js))
        .nest_service("/assets/images", ServeDir::new("wsta/assets/images"))
        .route("/status", get(status))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.http_bind).await?;
    log::info!("WSTA serving browser shell at http://{}/", cfg.http_bind);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index(State(_state): State<AppState>) -> Html<&'static str> {
    Html(crate::web::INDEX_HTML)
}

async fn css() -> impl IntoResponse {
    (
        [("content-type", "text/css; charset=utf-8")],
        crate::web::WSTA_CSS,
    )
}

async fn js(State(state): State<AppState>) -> impl IntoResponse {
    state.actor.notify_browser_connected().await;
    (
        [("content-type", "application/javascript; charset=utf-8")],
        crate::web::WSTA_JS,
    )
}

async fn status() -> impl IntoResponse {
    (
        [("content-type", "application/json; charset=utf-8")],
        r#"{"ok":true,"service":"wsta","role":"browser shell host only; controls use WebTransport"}"#,
    )
}
