// trade/wsta/src/main.rs
//
// wsta = browser-hosted replacement for the old Iced vsta frontend.
//
// Final shape:
//   - axum serves the browser/Makepad shell
//   - browser control transport is WebTransport-only
//   - actor owns frontend session state
//   - seek_bridge owns the existing dsta::ksta QUIC mesh connection
//   - dsta remains the protocol/type owner

mod actor;
mod protocol;
mod seek_bridge;
mod server;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cfg = server::WstaServerConfig::from_env();

    log::info!("WSTA starting");
    log::info!("WSTA HTTP bind: {}", cfg.http_bind);
    log::info!("WSTA seek backend addr: {}", cfg.seek_backend_addr);

    let actor = actor::spawn_actor(cfg.seek_backend_addr.clone());

    server::run(cfg, actor).await
}
