// trade/wsta_makepad/src/transport.rs
//
// Pure Makepad frontend transport boundary.
//
// Important:
// Makepad's web loader is its own runtime, not wasm-bindgen's generated JS
// loader. Do not use #[wasm_bindgen(...)] imports in this crate, because that
// creates a __wbindgen_placeholder__ import namespace that Makepad's loader does
// not provide.
//
// For this visual/frontend pass, transport never blocks rendering. It records
// packets locally so the UI can show/send/debug without a backend connection.
// Later, wire this to a Makepad-native browser bridge or a backend polling/
// WebTransport endpoint without adding wasm-bindgen imports.

use crate::protocol::BrowserToWsta;

#[derive(Debug, Clone)]
pub struct WstaTransport {
    pub last_sent_debug: String,
    pub queued_debug: Vec<String>,
}

impl Default for WstaTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl WstaTransport {
    pub fn new() -> Self {
        Self {
            last_sent_debug: "transport offline-safe; no packet sent yet".to_string(),
            queued_debug: Vec::new(),
        }
    }

    pub fn send(&mut self, pkt: &BrowserToWsta) -> Result<(), String> {
        let json = serde_json::to_string(pkt)
            .map_err(|e| format!("serialize BrowserToWsta failed: {}", e))?;

        self.last_sent_debug = json.clone();
        self.queued_debug.insert(0, json);
        self.queued_debug.truncate(64);

        Ok(())
    }

    pub fn status_line(&self) -> String {
        format!(
            "transport: offline-safe queue={} last={}",
            self.queued_debug.len(),
            self.last_sent_debug
        )
    }
}
