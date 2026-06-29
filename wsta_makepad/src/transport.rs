// trade/wsta_makepad/src/transport.rs

use crate::protocol::BrowserToWsta;

#[derive(Debug, Clone)]
pub struct WstaTransport {
    pub last_sent_debug: String,
}

impl Default for WstaTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl WstaTransport {
    pub fn new() -> Self {
        Self {
            last_sent_debug: "not sent yet".to_string(),
        }
    }

    pub fn send(&mut self, pkt: &BrowserToWsta) -> Result<(), String> {
        let json = serde_json::to_string(pkt)
            .map_err(|e| format!("serialize BrowserToWsta failed: {}", e))?;
        self.last_sent_debug = json.clone();

        #[cfg(target_arch = "wasm32")]
        {
            wasm_send_json(&json);
        }

        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(module = "/resources/web/wsta_transport.js")]
extern "C" {
    #[wasm_bindgen(js_name = wstaSendJson)]
    fn wasm_send_json(json: &str);
}
