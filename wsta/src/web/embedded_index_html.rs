// trade/wsta/src/web/embedded_index_html.rs
//
// Browser shell for the vsta replacement.
// Layout:
//   left  = navigation / image-button column
//   right = selected control view
//   bottom = debug/info output
//
// No video pane.
// No HTTP control fallback.
// WebTransport is the required browser control path.

pub const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <title>WSTA Trade Station</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" href="/assets/wsta.css">
</head>
<body>
  <div id="wstaRoot">
    <aside id="wstaNav">
      <div class="brand">WSTA</div>
      <div class="subbrand">trade web station</div>

      <button class="navButton imageButton selected" data-view="DrR">
        <img src="/assets/images/dr_robo_button.jpg" alt="Dr. Robotnik">
        <span>Dr. Robotnik</span>
      </button>
      <button class="navButton" data-view="Buzz">Buzz</button>
      <button class="navButton" data-view="Stealth">Stealth</button>
      <button class="navButton" data-view="Sally">Sally</button>
      <button class="navButton" data-view="Swat">Swat</button>
      <button class="navButton" data-view="Ttai">TTAI</button>
      <button class="navButton" data-view="Nico">Nico</button>
      <button class="navButton" data-view="Logs">Logs</button>

      <div class="navFooter">
        <div id="transportBadge" class="badge bad">WT disconnected</div>
      </div>
    </aside>

    <main id="wstaMain">
      <section id="wstaViewHeader">
        <h1 id="viewTitle">Dr. Robotnik</h1>
        <div id="viewSubtitle">main bot control surface</div>
      </section>

      <section id="wstaControlView">
        <div class="panel">
          <h2>Bot control</h2>

          <label>Bot name</label>
          <input id="botName" value="DRBOT_001">

          <div class="buttonRow">
            <button id="liveBot">Live</button>
            <button id="stopBot">Stop</button>
            <button id="killBot">Kill</button>
          </div>

          <label>Ticker</label>
          <input id="ticker" value="SPY">

          <div class="buttonRow">
            <button id="subscribeTicker">Subscribe ticker</button>
            <button id="reportStatus">Report status</button>
          </div>

          <label>Log note</label>
          <input id="logNote" value="hello from wsta">
          <button id="sendLogNote">Send log note</button>
        </div>

        <div class="panel">
          <h2>Selected view payload</h2>
          <pre id="viewPayload"></pre>
        </div>
      </section>
    </main>

    <footer id="wstaDebug">
      <div class="debugHeader">debug / backend info</div>
      <pre id="debugLog"></pre>
    </footer>
  </div>

  <script src="/assets/wsta.js"></script>
</body>
</html>
"#;

pub const WSTA_CSS: &str = r#"
:root {
  --bg: #05070b;
  --panel: #07101f;
  --panel2: #0b1428;
  --line: rgba(150, 190, 255, 0.24);
  --text: #dce7ff;
  --muted: #92a3c4;
  --hot: #70d6ff;
  --ok: #83f28f;
  --bad: #ff6b6b;
  --warn: #ffd166;
}

html,
body {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
  background:
    radial-gradient(circle at 50% 42%, rgba(112, 214, 255, 0.06), transparent 38%),
    linear-gradient(180deg, #02040a, #05070b);
  color: var(--text);
  font-family: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}

#wstaRoot {
  width: 100vw;
  height: 100vh;
  display: grid;
  grid-template-columns: 240px minmax(0, 1fr);
  grid-template-rows: minmax(0, 1fr) 190px;
}

#wstaNav {
  grid-row: 1 / 3;
  border-right: 1px solid var(--line);
  background: rgba(7, 16, 31, 0.94);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.brand {
  color: var(--hot);
  font-weight: 900;
  letter-spacing: 0.12em;
  font-size: 26px;
}

.subbrand {
  color: var(--muted);
  font-size: 12px;
  margin-bottom: 10px;
}

.navButton,
button {
  min-height: 36px;
  border: 1px solid rgba(112, 214, 255, 0.34);
  border-radius: 8px;
  background:
    linear-gradient(180deg, rgba(16, 32, 60, 0.92), rgba(7, 16, 31, 0.92));
  color: var(--text);
  cursor: pointer;
  font: 13px system-ui, sans-serif;
}

.navButton {
  width: 100%;
  text-align: left;
  padding-left: 12px;
}


.navButton.imageButton {
  min-height: 82px;
  padding: 8px;
  display: grid;
  grid-template-columns: 58px minmax(0, 1fr);
  align-items: center;
  gap: 10px;
}

.navButton.imageButton img {
  width: 58px;
  height: 58px;
  object-fit: contain;
  border-radius: 8px;
  border: 1px solid rgba(112, 214, 255, 0.24);
  background: rgba(2, 4, 10, 0.72);
}

.navButton.imageButton span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.navButton.selected {
  color: var(--hot);
  border-color: rgba(112, 214, 255, 0.82);
  box-shadow: 0 0 0 1px rgba(112, 214, 255, 0.22) inset;
}

.navFooter {
  margin-top: auto;
}

.badge {
  border: 1px solid var(--line);
  border-radius: 999px;
  padding: 6px 9px;
  font: 11px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.badge.ok {
  color: var(--ok);
  border-color: rgba(131, 242, 143, 0.40);
}

.badge.bad {
  color: var(--bad);
  border-color: rgba(255, 107, 107, 0.44);
}

#wstaMain {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: 74px minmax(0, 1fr);
}

#wstaViewHeader {
  border-bottom: 1px solid var(--line);
  background: rgba(6, 10, 22, 0.88);
  padding: 10px 16px;
}

#wstaViewHeader h1 {
  margin: 0;
  color: var(--hot);
  font-size: 24px;
}

#viewSubtitle {
  color: var(--muted);
  font-size: 12px;
}

#wstaControlView {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: 16px;
  display: grid;
  grid-template-columns: minmax(360px, 520px) minmax(0, 1fr);
  gap: 16px;
}

.panel {
  border: 1px solid var(--line);
  border-radius: 12px;
  background: rgba(7, 16, 31, 0.80);
  padding: 14px;
}

.panel h2 {
  margin-top: 0;
  color: var(--hot);
}

label {
  display: block;
  margin-top: 10px;
  margin-bottom: 4px;
  color: var(--muted);
  font-size: 12px;
}

input {
  box-sizing: border-box;
  width: 100%;
  min-height: 34px;
  padding: 7px 9px;
  border: 1px solid rgba(112, 214, 255, 0.34);
  border-radius: 7px;
  background: #02040a;
  color: var(--text);
  font: 13px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.buttonRow {
  display: flex;
  gap: 8px;
  margin-top: 8px;
}

.buttonRow button {
  flex: 1;
}

#wstaDebug {
  min-width: 0;
  min-height: 0;
  border-top: 1px solid var(--line);
  background: rgba(4, 9, 20, 0.96);
  display: grid;
  grid-template-rows: 28px minmax(0, 1fr);
}

.debugHeader {
  padding: 6px 10px;
  color: var(--hot);
  font-weight: 700;
  font-size: 12px;
  border-bottom: 1px solid rgba(112, 214, 255, 0.18);
}

pre {
  margin: 0;
  white-space: pre-wrap;
  overflow: auto;
  font: 11px/1.35 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

#debugLog {
  padding: 8px 10px;
  color: #aac8f0;
}

#viewPayload {
  padding: 10px;
  min-height: 180px;
  background: #02040a;
  border: 1px solid rgba(150, 190, 255, 0.18);
  border-radius: 8px;
}
"#;

pub const WSTA_JS: &str = r#"
(() => {
  const debugLog = document.getElementById("debugLog");
  const badge = document.getElementById("transportBadge");
  const viewTitle = document.getElementById("viewTitle");
  const viewSubtitle = document.getElementById("viewSubtitle");
  const viewPayload = document.getElementById("viewPayload");

  let wt = null;
  let writer = null;
  let selectedView = "DrR";

  function stamp() {
    return new Date().toLocaleTimeString();
  }

  function log(line) {
    debugLog.textContent = `[${stamp()}] ${line}\n` + debugLog.textContent;
  }

  function setBadge(ok, text) {
    badge.classList.toggle("ok", ok);
    badge.classList.toggle("bad", !ok);
    badge.textContent = text;
  }

  function viewLabel(view) {
    switch (view) {
      case "DrR": return ["Dr. Robotnik", "main bot control surface"];
      case "Buzz": return ["Buzz", "Buzz Bomber controls / chart view"];
      case "Stealth": return ["Stealth", "Stealth bot controls"];
      case "Sally": return ["Sally", "Sally fake-order controls"];
      case "Swat": return ["Swat", "SWAT bot controls"];
      case "Ttai": return ["TTAI", "account / positions / order updates"];
      case "Nico": return ["Nico", "assistant/chat control"];
      case "Logs": return ["Logs", "backend and frontend log notes"];
      default: return [view, "control view"];
    }
  }

  function renderSelectedView(view) {
    selectedView = view;

    for (const btn of document.querySelectorAll(".navButton")) {
      btn.classList.toggle("selected", btn.dataset.view === view);
    }

    const [title, subtitle] = viewLabel(view);
    viewTitle.textContent = title;
    viewSubtitle.textContent = subtitle;

    viewPayload.textContent = JSON.stringify({
      selectedView: view,
      nextPortStep: "port corresponding vsta::base/core/form/lens controls into this pane",
      currentTransport: "WebTransport only",
    }, null, 2);
  }

  function controlTransportUrl() {
    const params = new URLSearchParams(window.location.search);
    const explicit = params.get("controlTransport");
    if (explicit) return explicit;

    return "https://localhost:8089/transport/wsta-control";
  }

  async function connectWebTransport() {
    if (!("WebTransport" in window)) {
      setBadge(false, "WebTransport missing");
      throw new Error("Browser does not expose WebTransport");
    }

    const url = controlTransportUrl();
    log("connecting WebTransport " + url);

    wt = new WebTransport(url);
    await wt.ready;

    setBadge(true, "WT connected");
    log("WebTransport connected");

    const stream = await wt.createBidirectionalStream();
    writer = stream.writable.getWriter();

    readServerStream(stream.readable);

    sendPacket({kind: "SelectView", body: {view: selectedView}});
  }

  async function readServerStream(readable) {
    const reader = readable.getReader();
    const decoder = new TextDecoder();
    let buffered = "";

    while (true) {
      const {value, done} = await reader.read();
      if (done) break;

      buffered += decoder.decode(value, {stream: true});

      let idx;
      while ((idx = buffered.indexOf("\n")) >= 0) {
        const line = buffered.slice(0, idx).trim();
        buffered = buffered.slice(idx + 1);

        if (!line) continue;

        try {
          const msg = JSON.parse(line);
          handleServerMessage(msg);
        } catch (e) {
          log("bad server JSON: " + line);
        }
      }
    }

    setBadge(false, "WT closed");
    log("WebTransport server stream closed");
  }

  async function sendPacket(pkt) {
    if (!writer) {
      log("cannot send; WebTransport writer not ready");
      return;
    }

    const line = JSON.stringify(pkt) + "\n";
    const bytes = new TextEncoder().encode(line);

    await writer.write(bytes);
    log("SEND " + JSON.stringify(pkt));
  }

  function handleServerMessage(msg) {
    log("RECV " + JSON.stringify(msg));

    if (msg.kind === "SelectedView") {
      renderSelectedView(msg.body.view);
    }
  }

  function inputValue(id) {
    return document.getElementById(id).value;
  }

  for (const btn of document.querySelectorAll(".navButton")) {
    btn.addEventListener("click", () => {
      const view = btn.dataset.view;
      renderSelectedView(view);
      sendPacket({kind: "SelectView", body: {view}});
    });
  }

  document.getElementById("liveBot").addEventListener("click", () => {
    sendPacket({kind: "LiveBotName", body: {name: inputValue("botName")}});
  });

  document.getElementById("stopBot").addEventListener("click", () => {
    sendPacket({kind: "StopBotName", body: {name: inputValue("botName")}});
  });

  document.getElementById("killBot").addEventListener("click", () => {
    sendPacket({kind: "KillBotName", body: {name: inputValue("botName")}});
  });

  document.getElementById("subscribeTicker").addEventListener("click", () => {
    sendPacket({kind: "SubscribeToTicker", body: {ticker: inputValue("ticker")}});
  });

  document.getElementById("reportStatus").addEventListener("click", () => {
    sendPacket({kind: "ReportOfAllStatus"});
  });

  document.getElementById("sendLogNote").addEventListener("click", () => {
    sendPacket({kind: "SendLogNote", body: {text: inputValue("logNote")}});
  });

  renderSelectedView("DrR");

  connectWebTransport().catch((e) => {
    setBadge(false, "WT failed");
    log("FATAL WebTransport failure: " + e.message);
  });
})();
"#;
