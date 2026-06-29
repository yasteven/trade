// trade/wsta/src/web/embedded_index_html.rs
//
// Final browser shell for WSTA.
// This is not the UI.
// It only loads the Makepad/WASM frontend and its WebTransport adapter.
//
// Important runtime behavior:
// - if /makepad/wsta_makepad.js is not built yet, keep retrying
// - do not permanently fail the browser page
// - do not add HTML controls here; controls live in wsta_makepad Rust/Makepad

pub const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>WSTA Makepad</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <style>
    html, body {
      margin: 0;
      width: 100%;
      height: 100%;
      overflow: hidden;
      background: #05070b;
      color: #dce7ff;
    }

    #makepad_frame {
      width: 100vw;
      height: 100vh;
      border: 0;
      display: block;
      background: #05070b;
    }

    #status_panel {
      position: fixed;
      left: 10px;
      bottom: 10px;
      z-index: 1000;
      color: #70d6ff;
      background: rgba(5, 7, 11, 0.84);
      border: 1px solid rgba(112, 214, 255, 0.34);
      border-radius: 6px;
      padding: 7px 9px;
      font: 11px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      max-width: calc(100vw - 24px);
      white-space: pre-wrap;
      pointer-events: none;
    }

    #fallback_ui {
      position: fixed;
      inset: 0;
      z-index: 1;
      display: grid;
      place-items: center;
      background:
        radial-gradient(circle at 30% 20%, rgba(112,214,255,0.13), transparent 25%),
        radial-gradient(circle at 70% 80%, rgba(255,209,102,0.09), transparent 28%),
        #05070b;
      color: #dce7ff;
      font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
    }

    #fallback_card {
      width: min(760px, calc(100vw - 32px));
      border: 1px solid rgba(112, 214, 255, 0.22);
      background: rgba(7, 16, 31, 0.78);
      border-radius: 10px;
      padding: 18px;
      box-shadow: 0 12px 40px rgba(0,0,0,0.36);
    }

    #fallback_card h1 {
      margin: 0 0 8px 0;
      font-size: 26px;
      color: #70d6ff;
    }

    #fallback_card p {
      margin: 6px 0;
      color: #aac8f0;
      line-height: 1.35;
    }

    #fallback_card code {
      color: #ffd166;
    }
  </style>
</head>
<body>
  <div id="fallback_ui">
    <div id="fallback_card">
      <h1>WSTA</h1>
      <p>Host page is alive. The Makepad package is loaded below when <code>/makepad/index.html</code> is reachable.</p>
      <p>The backend/seek connection is not required for the GUI to display. Connection state belongs in the status panel and Makepad debug area.</p>
      <p>Current backend path is still retry/offline until the WebTransport bridge is attached.</p>
    </div>
  </div>

  <iframe id="makepad_frame" title="WSTA Makepad"></iframe>

  <div id="status_panel">WSTA shell booting...</div>

  <script type="module">
    const frame = document.getElementById("makepad_frame");
    const statusPanel = document.getElementById("status_panel");
    const fallback = document.getElementById("fallback_ui");

    const makepadIndex = "/makepad/index.html";
    const statusUrl = "/status";
    const retryMs = 3000;

    let frameLoaded = false;
    let frameStarted = false;
    let hostOk = false;
    let packageOk = false;
    let lastError = "none yet";

    function setStatus() {
      statusPanel.textContent =
        "WSTA host: " + (hostOk ? "ok" : "checking") + "\n" +
        "Makepad package: " + (packageOk ? "reachable" : "waiting") + "\n" +
        "Makepad frame: " + (frameLoaded ? "loaded" : (frameStarted ? "loading" : "not started")) + "\n" +
        "backend transport: retrying/offline-safe\n" +
        "last: " + lastError;
    }

    async function probe(url) {
      const res = await fetch(url + (url.includes("?") ? "&" : "?") + "probe=" + Date.now(), {
        method: "GET",
        cache: "no-store"
      });
      return res.ok ? res : Promise.reject(new Error(url + " HTTP " + res.status));
    }

    frame.addEventListener("load", () => {
      frameLoaded = true;
      lastError = "Makepad iframe load event fired";
      fallback.style.display = "none";
      setStatus();
    });

    frame.addEventListener("error", () => {
      frameLoaded = false;
      lastError = "Makepad iframe error event fired";
      fallback.style.display = "grid";
      setStatus();
    });

    async function runForever() {
      while (true) {
        try {
          await probe(statusUrl);
          hostOk = true;
        } catch (e) {
          hostOk = false;
          lastError = e && e.message ? e.message : String(e);
        }

        try {
          await probe(makepadIndex);
          packageOk = true;

          if (!frameStarted) {
            frameStarted = true;
            frame.src = makepadIndex + "?wsta_load=" + Date.now();
            lastError = "started Makepad frame";
          }
        } catch (e) {
          packageOk = false;
          frameStarted = false;
          frameLoaded = false;
          frame.removeAttribute("src");
          fallback.style.display = "grid";
          lastError = e && e.message ? e.message : String(e);
        }

        setStatus();
        await new Promise(resolve => setTimeout(resolve, retryMs));
      }
    }

    setStatus();
    runForever();
  </script>
</body>
</html>
"#;
