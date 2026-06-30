// trade/wsta/src/server.rs
//
// Pure Makepad frontend host.
//
// wsta serves the generated Makepad/WASM package directly at /.
// Browser-visible controls live in wsta_makepad Rust/Makepad.
// Backend/seek can be offline while the UI still renders.
//
// HTTP responsibilities:
//   - serve Makepad generated index/assets at /
//   - serve /resources for optional app resources
//   - serve /status for host diagnostics
//   - receive /$report_error from generated/index-injected browser profiling
//   - add COOP/COEP headers required by Makepad WASM/thread/shared-memory runtime

use crate::actor::ActorHandle;
use axum::{
    body::Body,
    extract::{Query, State},
    http::{HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use std::collections::HashMap;
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

    let makepad_dir = "wsta_makepad/target/makepad-wasm";

    let app = Router::new()
        .route("/", get(index))
        .route("/status", get(status))
        .route("/$report_error", get(report_error))
        .nest_service("/resources", ServeDir::new("wsta_makepad/resources"))
        .nest_service("/makepad_resources", ServeDir::new("wsta_makepad/resources"))
        .nest_service("/assets/images", ServeDir::new("wsta/assets/images"))
        .nest_service("/makepad", ServeDir::new(makepad_dir))
        .fallback_service(ServeDir::new(makepad_dir))
        .layer(CorsLayer::permissive())
        .layer(middleware::from_fn(add_browser_runtime_headers))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&cfg.http_bind).await?;
    log::info!("WSTA serving pure Makepad frontend at http://{}/", cfg.http_bind);
    log::info!("WSTA generated Makepad package dir: {}", makepad_dir);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn add_browser_runtime_headers(
    request: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

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
    headers.insert(
        "Cache-Control",
        HeaderValue::from_static("no-store"),
    );

    response
}

async fn index(State(_state): State<AppState>) -> impl IntoResponse {
    let path = std::path::Path::new("wsta_makepad/target/makepad-wasm/index.html");

    match tokio::fs::read_to_string(path).await {
        Ok(html) => Html(html).into_response(),
        Err(e) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Html(format!(
                r#"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>WSTA Makepad not built</title>
  <style>
    html, body {{
      margin: 0;
      width: 100%;
      height: 100%;
      background: #05070b;
      color: #dce7ff;
      font: 14px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      display: grid;
      place-items: center;
    }}
    main {{
      width: min(860px, calc(100vw - 32px));
      border: 1px solid rgba(112,214,255,0.28);
      border-radius: 10px;
      background: rgba(7,16,31,0.82);
      padding: 20px;
    }}
    h1 {{ color: #70d6ff; margin-top: 0; }}
    code {{ color: #ffd166; }}
  </style>
</head>
<body>
  <main>
    <h1>WSTA Makepad package is not built/deployed</h1>
    <p>The pure Makepad frontend is served directly from:</p>
    <p><code>wsta_makepad/target/makepad-wasm/index.html</code></p>
    <p>Build/deploy it with:</p>
    <p><code>./tools/build_wsta_makepad_wasm.sh</code></p>
    <p>Last read error: <code>{}</code></p>
  </main>
</body>
</html>"#,
                e
            )),
        )
            .into_response(),
    }
}

async fn status() -> impl IntoResponse {
    (
        [("content-type", "application/json; charset=utf-8")],
        r#"{"ok":true,"service":"wsta","role":"pure Makepad frontend host; backend seek connection may be retrying"}"#,
    )
}

async fn report_error(Query(params): Query<HashMap<String, String>>) -> impl IntoResponse {
    if let Some(data) = params.get("data") {
        log::warn!("WSTA browser report: {}", data);
    } else {
        log::warn!("WSTA browser report without data");
    }

    (
        [("content-type", "application/json; charset=utf-8")],
        r#"{"ok":true}"#,
    )
}
