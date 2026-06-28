// trade/wsta/src/web/embedded_index_html.rs
//
// Browser shell for the vsta replacement.
// Dr. Robotnik is the first rebuilt control surface.
//
// Layout:
//   global left    = app nav image buttons
//   main/right     = selected control view
//   bottom         = debug/info output
//
// Dr. Robotnik selected:
//   inner left     = thin bot-maker options
//   inner right    = selected form/display
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
        <div id="viewSubtitle">bot constructor and backend overview</div>
      </section>

      <section id="wstaControlView"></section>
    </main>

    <footer id="wstaDebug">
      <div class="debugHeader">debug / backend info</div>
      <pre id="debugLog"></pre>
    </footer>
  </div>

  <template id="drRobotnikTemplate">
    <div id="drRobotnikSurface">
      <nav id="drRobotnikTools">
        <button class="drTool selected" data-dr-tool="Overview">Overview</button>
        <button class="drTool" data-dr-tool="MakeBuzz">Make Buzz</button>
        <button class="drTool" data-dr-tool="MakeStealth">Make Stealth</button>
        <button class="drTool" data-dr-tool="MakeSally">Make Sally</button>
        <button class="drTool" data-dr-tool="MakeSwat">Make Swat</button>
        <button class="drTool" data-dr-tool="TtaiOverview">TTAI Overview</button>
      </nav>

      <section id="drRobotnikDisplay"></section>
    </div>
  </template>

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

.navButton.selected,
.drTool.selected {
  color: var(--hot);
  border-color: rgba(112, 214, 255, 0.82);
  box-shadow: 0 0 0 1px rgba(112, 214, 255, 0.22) inset;
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
  overflow: hidden;
}

#drRobotnikSurface {
  width: 100%;
  height: 100%;
  display: grid;
  grid-template-columns: 138px minmax(0, 1fr);
  gap: 14px;
  box-sizing: border-box;
  padding: 14px;
}

#drRobotnikTools {
  min-width: 0;
  min-height: 0;
  border: 1px solid var(--line);
  border-radius: 12px;
  background: rgba(7, 16, 31, 0.80);
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.drTool {
  min-height: 46px;
  padding: 6px 8px;
  text-align: left;
  font-size: 12px;
}

#drRobotnikDisplay {
  min-width: 0;
  min-height: 0;
  overflow: auto;
}

.panelGrid {
  display: grid;
  grid-template-columns: minmax(340px, 520px) minmax(0, 1fr);
  gap: 14px;
  align-items: start;
}

.panel {
  border: 1px solid var(--line);
  border-radius: 12px;
  background: rgba(7, 16, 31, 0.80);
  padding: 14px;
}

.panel h2,
.panel h3 {
  margin-top: 0;
  color: var(--hot);
}

.infoCards {
  display: grid;
  grid-template-columns: repeat(2, minmax(180px, 1fr));
  gap: 10px;
}

.infoCard {
  border: 1px solid rgba(150, 190, 255, 0.18);
  border-radius: 10px;
  padding: 10px;
  background: rgba(2, 4, 10, 0.52);
}

.infoCard .label {
  color: var(--muted);
  font-size: 11px;
}

.infoCard .value {
  color: var(--text);
  margin-top: 4px;
  font: 13px ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.formGrid {
  display: grid;
  grid-template-columns: 170px minmax(0, 1fr);
  gap: 8px 10px;
  align-items: center;
}

.formGrid label {
  color: var(--muted);
  font-size: 12px;
}

input,
select {
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

.checkLine {
  display: flex;
  gap: 8px;
  align-items: center;
}

.checkLine input {
  width: auto;
  min-height: auto;
}

.buttonRow {
  display: flex;
  gap: 8px;
  margin-top: 12px;
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

.payloadPre {
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
  const controlView = document.getElementById("wstaControlView");

  let wt = null;
  let writer = null;
  let selectedView = "DrR";
  let selectedDrTool = "Overview";

  const backendState = {
    madeBots: [],
    account: null,
    positions: null,
    sentOrders: [],
    orderUpdates: [],
    pushTickers: {},
  };

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
      case "DrR": return ["Dr. Robotnik", "bot constructor and backend overview"];
      case "Buzz": return ["Buzz", "Buzz Bomber live view"];
      case "Stealth": return ["Stealth", "Stealth bot live view"];
      case "Sally": return ["Sally", "Sally fake-order live view"];
      case "Swat": return ["Swat", "SWAT bot live view"];
      case "Ttai": return ["TTAI", "account / positions / order updates"];
      case "Nico": return ["Nico", "assistant/chat control"];
      case "Logs": return ["Logs", "backend and frontend log notes"];
      default: return [view, "control view"];
    }
  }

  function setHeader(view) {
    const [title, subtitle] = viewLabel(view);
    viewTitle.textContent = title;
    viewSubtitle.textContent = subtitle;
  }

  function html(strings, ...values) {
    return strings.reduce((out, s, i) => out + s + (values[i] ?? ""), "");
  }

  function escapeHtml(x) {
    return String(x)
      .replaceAll("&", "&amp;")
      .replaceAll("<", "&lt;")
      .replaceAll(">", "&gt;")
      .replaceAll('"', "&quot;");
  }

  function formValue(id) {
    return document.getElementById(id).value;
  }

  function formNumber(id) {
    const x = Number(formValue(id));
    return Number.isFinite(x) ? x : 0;
  }

  function formChecked(id) {
    return document.getElementById(id).checked;
  }

  function commonForm(prefix, name, ticker) {
    return html`
      <label>Friendly name</label>
      <input id="${prefix}_friendlyName" value="${escapeHtml(name)}">

      <label>Tracking ticker</label>
      <input id="${prefix}_trackingTick" value="${escapeHtml(ticker)}">
    `;
  }

  function marketDirectionSelect(id) {
    return html`
      <select id="${id}">
        <option value="GetStonk">GetStonk</option>
        <option value="Sideways">Sideways</option>
        <option value="Corrects">Corrects</option>
      </select>
    `;
  }

  function renderDrRobotnikSurface() {
    const template = document.getElementById("drRobotnikTemplate");
    controlView.replaceChildren(template.content.cloneNode(true));

    for (const btn of controlView.querySelectorAll(".drTool")) {
      btn.classList.toggle("selected", btn.dataset.drTool === selectedDrTool);
      btn.addEventListener("click", () => {
        selectedDrTool = btn.dataset.drTool;
        renderDrTool();
      });
    }

    renderDrTool();
  }

  function renderDrTool() {
    for (const btn of controlView.querySelectorAll(".drTool")) {
      btn.classList.toggle("selected", btn.dataset.drTool === selectedDrTool);
    }

    const display = document.getElementById("drRobotnikDisplay");

    switch (selectedDrTool) {
      case "MakeBuzz":
        display.innerHTML = renderMakeBuzz();
        wireMakeBuzz();
        break;

      case "MakeStealth":
        display.innerHTML = renderMakeStealth();
        wireMakeStealth();
        break;

      case "MakeSally":
        display.innerHTML = renderMakeSally();
        wireMakeSally();
        break;

      case "MakeSwat":
        display.innerHTML = renderMakeSwat();
        wireMakeSwat();
        break;

      case "TtaiOverview":
        display.innerHTML = renderTtaiOverview();
        break;

      case "Overview":
      default:
        display.innerHTML = renderDrOverview();
        break;
    }
  }

  function renderDrOverview() {
    return html`
      <div class="panelGrid">
        <div class="panel">
          <h2>Dr. Robotnik overview</h2>
          <p>This is the web rebuild of the old Iced <code>DrRobotnikV</code> hub.</p>
          <p>The thin column selects bot-maker tools. The display column renders the selected control.</p>

          <div class="infoCards">
            <div class="infoCard">
              <div class="label">connection</div>
              <div class="value">${writer ? "WebTransport ready" : "WebTransport not ready"}</div>
            </div>
            <div class="infoCard">
              <div class="label">made bots</div>
              <div class="value">${backendState.madeBots.length}</div>
            </div>
            <div class="infoCard">
              <div class="label">sent orders</div>
              <div class="value">${backendState.sentOrders.length}</div>
            </div>
            <div class="infoCard">
              <div class="label">push tickers</div>
              <div class="value">${Object.keys(backendState.pushTickers).length}</div>
            </div>
          </div>
        </div>

        <div class="panel">
          <h2>Backend snapshot</h2>
          <pre class="payloadPre">${escapeHtml(JSON.stringify(backendState, null, 2))}</pre>
        </div>
      </div>
    `;
  }

  function renderMakeBuzz() {
    return html`
      <div class="panelGrid">
        <div class="panel">
          <h2>Make Buzz Bomber</h2>
          <div class="formGrid">
            ${commonForm("buzz", "Buzz Bot 1", "SPY")}

            <label>Cash alloc</label>
            <input id="buzz_cashAlloc" value="150">

            <label>Market direction</label>
            ${marketDirectionSelect("buzz_marketDirection")}

            <label>Option expire days</label>
            <input id="buzz_optionExpire" value="5">

            <label>Target spread</label>
            <input id="buzz_targetSpread" value="0.25">

            <label>Bombs forever</label>
            <div class="checkLine">
              <input id="buzz_bombsForever" type="checkbox" checked>
              <span>repeat after resets</span>
            </div>
          </div>

          <div class="buttonRow">
            <button id="buzz_create">Create Buzz Bot</button>
          </div>
        </div>

        <div class="panel">
          <h2>Payload</h2>
          <pre id="buzz_payload" class="payloadPre"></pre>
        </div>
      </div>
    `;
  }

  function renderMakeStealth() {
    return html`
      <div class="panelGrid">
        <div class="panel">
          <h2>Make Stealth Bot</h2>
          <div class="formGrid">
            ${commonForm("stealth", "Stealth Bot 1", "SPY")}

            <label>Cash alloc</label>
            <input id="stealth_cashAlloc" value="150">

            <label>Market direction</label>
            ${marketDirectionSelect("stealth_marketDirection")}

            <label>Option expire days</label>
            <input id="stealth_optionExpire" value="5">

            <label>Option bucket</label>
            <input id="stealth_optionBucket" value="0">

            <label>Spread bucket</label>
            <input id="stealth_spreadBucket" value="1">

            <label>Exit gain %</label>
            <input id="stealth_exitGainPct" value="50">

            <label>Exit loss %</label>
            <input id="stealth_exitLossPct" value="-50">

            <label>Use theo cost</label>
            <div class="checkLine">
              <input id="stealth_useTheoCost" type="checkbox">
              <span>use theoretical cost</span>
            </div>
          </div>

          <div class="buttonRow">
            <button id="stealth_create">Create Stealth Bot</button>
          </div>
        </div>

        <div class="panel">
          <h2>Payload</h2>
          <pre id="stealth_payload" class="payloadPre"></pre>
        </div>
      </div>
    `;
  }

  function renderMakeSally() {
    return html`
      <div class="panelGrid">
        <div class="panel">
          <h2>Make Sally Fakes</h2>
          <p>First pass restores the Dr. Robotnik routing and accounting shell. Order-leg/reveal-way controls come next.</p>
          <div class="formGrid">
            ${commonForm("sally", "Sally Bot 1", "SPY")}
          </div>

          <div class="buttonRow">
            <button id="sally_create">Create Sally Bot</button>
          </div>
        </div>

        <div class="panel">
          <h2>Payload</h2>
          <pre id="sally_payload" class="payloadPre"></pre>
        </div>
      </div>
    `;
  }

  function renderMakeSwat() {
    return html`
      <div class="panelGrid">
        <div class="panel">
          <h2>Make Swat Bot</h2>
          <p>First pass restores the Dr. Robotnik routing and accounting shell. Emerald/asset selection comes next.</p>
          <div class="formGrid">
            ${commonForm("swat", "Swat Bot 1", "SPY")}
          </div>

          <div class="buttonRow">
            <button id="swat_create">Create Swat Bot</button>
          </div>
        </div>

        <div class="panel">
          <h2>Payload</h2>
          <pre id="swat_payload" class="payloadPre"></pre>
        </div>
      </div>
    `;
  }

  function renderTtaiOverview() {
    return html`
      <div class="panelGrid">
        <div class="panel">
          <h2>TTAI overview</h2>
          <div class="infoCards">
            <div class="infoCard"><div class="label">account</div><div class="value">${backendState.account ? "loaded" : "none"}</div></div>
            <div class="infoCard"><div class="label">positions</div><div class="value">${backendState.positions ? "loaded" : "none"}</div></div>
            <div class="infoCard"><div class="label">sent orders</div><div class="value">${backendState.sentOrders.length}</div></div>
            <div class="infoCard"><div class="label">order updates</div><div class="value">${backendState.orderUpdates.length}</div></div>
          </div>

          <div class="buttonRow">
            <button id="ttai_report">Report all status</button>
          </div>
        </div>

        <div class="panel">
          <h2>Raw TTAI state</h2>
          <pre class="payloadPre">${escapeHtml(JSON.stringify({
            account: backendState.account,
            positions: backendState.positions,
            sentOrders: backendState.sentOrders,
            orderUpdates: backendState.orderUpdates,
            pushTickers: backendState.pushTickers,
          }, null, 2))}</pre>
        </div>
      </div>
    `;
  }

  function wirePayloadPreview(prefix, buildPacket) {
    const ids = Array.from(controlView.querySelectorAll("input, select")).map(x => x.id);
    const update = () => {
      const el = document.getElementById(`${prefix}_payload`);
      if (el) el.textContent = JSON.stringify(buildPacket(), null, 2);
    };
    for (const id of ids) {
      const el = document.getElementById(id);
      if (el) {
        el.addEventListener("input", update);
        el.addEventListener("change", update);
      }
    }
    update();
  }

  function buildBuzzPacket() {
    return {
      kind: "CreateBuzzBot",
      body: {
        friendly_name: formValue("buzz_friendlyName"),
        tracking_tick: formValue("buzz_trackingTick"),
        cash_alloc: formNumber("buzz_cashAlloc"),
        market_direction: formValue("buzz_marketDirection"),
        option_expire: formNumber("buzz_optionExpire"),
        target_spread: formNumber("buzz_targetSpread"),
        bombs_forever: formChecked("buzz_bombsForever"),
      },
    };
  }

  function wireMakeBuzz() {
    wirePayloadPreview("buzz", buildBuzzPacket);
    document.getElementById("buzz_create").addEventListener("click", () => sendPacket(buildBuzzPacket()));
  }

  function buildStealthPacket() {
    return {
      kind: "CreateStealthBot",
      body: {
        friendly_name: formValue("stealth_friendlyName"),
        tracking_tick: formValue("stealth_trackingTick"),
        cash_alloc: formNumber("stealth_cashAlloc"),
        market_direction: formValue("stealth_marketDirection"),
        option_expire: formNumber("stealth_optionExpire"),
        option_bucket: formNumber("stealth_optionBucket"),
        spread_bucket: formNumber("stealth_spreadBucket"),
        exit_gain_pct: formNumber("stealth_exitGainPct"),
        exit_loss_pct: formNumber("stealth_exitLossPct"),
        use_theo_cost: formChecked("stealth_useTheoCost"),
      },
    };
  }

  function wireMakeStealth() {
    wirePayloadPreview("stealth", buildStealthPacket);
    document.getElementById("stealth_create").addEventListener("click", () => sendPacket(buildStealthPacket()));
  }

  function buildSallyPacket() {
    return {
      kind: "CreateSallyBot",
      body: {
        friendly_name: formValue("sally_friendlyName"),
        tracking_tick: formValue("sally_trackingTick"),
      },
    };
  }

  function wireMakeSally() {
    wirePayloadPreview("sally", buildSallyPacket);
    document.getElementById("sally_create").addEventListener("click", () => sendPacket(buildSallyPacket()));
  }

  function buildSwatPacket() {
    return {
      kind: "CreateSwatBot",
      body: {
        friendly_name: formValue("swat_friendlyName"),
        tracking_tick: formValue("swat_trackingTick"),
      },
    };
  }

  function wireMakeSwat() {
    wirePayloadPreview("swat", buildSwatPacket);
    document.getElementById("swat_create").addEventListener("click", () => sendPacket(buildSwatPacket()));
  }

  function renderGenericView(view) {
    controlView.innerHTML = html`
      <div style="padding:14px">
        <div class="panel">
          <h2>${escapeHtml(view)}</h2>
          <p>This live view will be rebuilt after the Dr. Robotnik constructor hub.</p>
          <pre class="payloadPre">${escapeHtml(JSON.stringify({selectedView: view}, null, 2))}</pre>
        </div>
      </div>
    `;
  }

  function renderSelectedView(view) {
    selectedView = view;

    for (const btn of document.querySelectorAll(".navButton")) {
      btn.classList.toggle("selected", btn.dataset.view === view);
    }

    setHeader(view);

    if (view === "DrR") {
      renderDrRobotnikSurface();
    } else {
      renderGenericView(view);
    }
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
    renderSelectedView(selectedView);
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
        } catch (_e) {
          log("bad server JSON: " + line);
        }
      }
    }

    setBadge(false, "WT closed");
    log("WebTransport server stream closed");
  }

  async function sendPacket(pkt) {
    if (!writer) {
      log("cannot send; WebTransport writer not ready: " + JSON.stringify(pkt));
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
      return;
    }

    if (msg.kind === "BackendMadeBot") {
      backendState.madeBots.unshift(msg.body.value);
      renderSelectedView(selectedView);
      return;
    }

    if (msg.kind === "BackendGotTtai") {
      const value = msg.body.value;
      if (value.NewAccountUpdate) backendState.account = value.NewAccountUpdate;
      if (value.NewPositionsList) backendState.positions = value.NewPositionsList;
      if (value.NewPushOrderTask) backendState.sentOrders.unshift(value.NewPushOrderTask);
      if (value.NewOrderToUpdate) backendState.orderUpdates.unshift(value.NewOrderToUpdate);
      if (value.NewPushTickerRes) {
        const key = JSON.stringify(value.NewPushTickerRes.ass_deets ?? value.NewPushTickerRes);
        backendState.pushTickers[key] = value.NewPushTickerRes;
      }
      renderSelectedView(selectedView);
      return;
    }
  }

  for (const btn of document.querySelectorAll(".navButton")) {
    btn.addEventListener("click", () => {
      const view = btn.dataset.view;
      renderSelectedView(view);
      sendPacket({kind: "SelectView", body: {view}});
    });
  }

  renderSelectedView("DrR");

  connectWebTransport().catch((e) => {
    setBadge(false, "WT failed");
    log("FATAL WebTransport failure: " + e.message);
  });
})();
"#;
