// trade/wsta_makepad/src/lib.rs
//
// Final browser frontend crate.
// This is the replacement for the old Iced vsta frontend.
//
// UI: Makepad / Rust / WASM
// Transport: WebTransport browser adapter
// Protocol: dsta-owned bot/control packets

pub mod app;
pub mod protocol;
pub mod transport;

pub use app::app_main;
