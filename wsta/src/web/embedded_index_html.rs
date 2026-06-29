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
  <link rel="modulepreload" href="/makepad/wsta_transport.js">
  <style>
    html, body {
      margin: 0;
      width: 100%;
      height: 100%;
      overflow: hidden;
      background: #05070b;
    }

    #makepad_app {
      width: 100vw;
      height: 100vh;
      display: block;
    }

    #boot_status {
      position: fixed;
      left: 10px;
      bottom: 10px;
      z-index: 1000;
      color: #70d6ff;
      background: rgba(5, 7, 11, 0.78);
      border: 1px solid rgba(112, 214, 255, 0.34);
      border-radius: 6px;
      padding: 6px 8px;
      font: 11px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
      pointer-events: none;
      max-width: calc(100vw - 24px);
      white-space: pre-wrap;
    }
  </style>
</head>
<body>
  <canvas id="makepad_app"></canvas>
  <div id="boot_status">waiting for wsta_makepad wasm artifact</div>

  <script type="module">
    import "/makepad/wsta_transport.js";

    const boot = document.getElementById("boot_status");

    const makepadCandidates = [
      "/makepad/wsta_makepad.js",
      "/makepad/pkg/wsta_makepad.js",
      "/makepad/wsta_makepad/wsta_makepad.js"
    ];

    let booted = false;
    let attempt = 0;

    function sleep(ms) {
      return new Promise(resolve => setTimeout(resolve, ms));
    }

    async function tryBootCandidate(url) {
      const cacheBust = url.includes("?") ? "&" : "?";
      const mod = await import(url + cacheBust + "wsta_retry=" + Date.now());

      if (typeof mod.default === "function") {
        await mod.default();
        return true;
      }

      if (typeof mod.main === "function") {
        await mod.main();
        return true;
      }

      if (typeof mod.start === "function") {
        await mod.start();
        return true;
      }

      throw new Error(url + " loaded but exported no default/main/start function");
    }

    async function bootMakepadForever() {
      while (!booted) {
        attempt += 1;
        let lastError = null;

        for (const url of makepadCandidates) {
          try {
            boot.textContent = "loading Makepad/WASM attempt " + attempt + "\n" + url;
            booted = await tryBootCandidate(url);

            if (booted) {
              boot.textContent = "wsta_makepad wasm loaded";
              setTimeout(() => boot.remove(), 1200);
              return;
            }
          } catch (e) {
            lastError = e;
          }
        }

        const detail = lastError && lastError.message ? lastError.message : String(lastError);
        console.warn("wsta_makepad wasm not ready; retrying", detail);
        boot.textContent =
          "waiting for Makepad/WASM artifact; retrying in 3s\n" +
          "expected one of:\n" +
          makepadCandidates.join("\n") +
          "\nlast error: " + detail;

        await sleep(3000);
      }
    }

    bootMakepadForever();
  </script>
</body>
</html>
"#;
