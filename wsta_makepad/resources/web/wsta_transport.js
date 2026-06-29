// trade/wsta_makepad/resources/web/wsta_transport.js
//
// Browser API adapter for the Makepad/WASM frontend.
// This is not UI code. Makepad owns the UI.
// This adapter owns WebTransport because WebTransport is a browser API.
//
// Runtime behavior:
// - queue outbound packets until connected
// - retry connection forever every few seconds
// - if the stream closes, reconnect and keep queued packets

let wstaTransport = null;
let wstaWriter = null;
let pending = [];
let connecting = false;
let reconnectTimer = null;
let connectAttempt = 0;

function defaultTransportUrl() {
  const params = new URLSearchParams(window.location.search);
  const explicit = params.get("controlTransport");
  if (explicit) return explicit;
  return "https://localhost:8089/transport/wsta-control";
}

function note(...args) {
  console.log("wsta transport:", ...args);
}

function warn(...args) {
  console.warn("wsta transport:", ...args);
}

function clearWriter() {
  try {
    if (wstaWriter) wstaWriter.releaseLock();
  } catch (_e) {}

  wstaWriter = null;
  wstaTransport = null;
}

function scheduleReconnect(reason) {
  clearWriter();

  if (reason) warn("reconnect scheduled:", reason);

  if (reconnectTimer) return;

  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    connectWstaTransport();
  }, 3000);
}

async function flushPending() {
  if (!wstaWriter) return;

  while (pending.length > 0 && wstaWriter) {
    const line = pending.shift();
    await wstaWriter.write(new TextEncoder().encode(line + "\n"));
  }
}

async function connectWstaTransport() {
  if (connecting) return;

  if (!("WebTransport" in window)) {
    scheduleReconnect("WebTransport is not available in this browser");
    return;
  }

  connecting = true;
  connectAttempt += 1;

  const url = defaultTransportUrl();

  try {
    note("connecting", url, "attempt", connectAttempt);

    const transport = new WebTransport(url);
    await transport.ready;

    const stream = await transport.createBidirectionalStream();
    const writer = stream.writable.getWriter();

    wstaTransport = transport;
    wstaWriter = writer;

    note("connected", url);

    readWstaTransport(stream.readable).catch((e) => {
      scheduleReconnect("read loop failed: " + (e && e.message ? e.message : String(e)));
    });

    transport.closed
      .then(() => scheduleReconnect("transport closed"))
      .catch((e) => scheduleReconnect("transport closed with error: " + (e && e.message ? e.message : String(e))));

    await flushPending();
  } catch (e) {
    scheduleReconnect(e && e.message ? e.message : String(e));
  } finally {
    connecting = false;
  }
}

async function readWstaTransport(readable) {
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

      if (line) {
        console.log("wsta backend -> makepad", line);
      }
    }
  }

  scheduleReconnect("server stream closed");
}

export function wstaSendJson(json) {
  if (typeof json !== "string") {
    json = JSON.stringify(json);
  }

  if (!wstaWriter) {
    pending.push(json);
    connectWstaTransport();
    return;
  }

  wstaWriter
    .write(new TextEncoder().encode(json + "\n"))
    .catch((e) => {
      pending.unshift(json);
      scheduleReconnect("write failed: " + (e && e.message ? e.message : String(e)));
    });
}

connectWstaTransport();
