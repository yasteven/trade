// trade/wsta_makepad/src/lib.rs
//
// Final browser frontend crate for WSTA.
// Makepad owns the UI. JS only adapts browser WebTransport.

pub mod app;
pub mod protocol;
pub mod transport;

pub use app::app_main;
