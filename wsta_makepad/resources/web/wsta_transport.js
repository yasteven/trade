// trade/wsta_makepad/resources/web/wsta_transport.js
// Browser WebTransport adapter. Not UI.

let wstaTransport = null;
let wstaWriter = null;
let pending = [];
let connecting = false;
const RETRY_MS = 3000;

function defaultTransportUrl() {
  const params = new URLSearchParams(window.location.search);
  const explicit = params.get("controlTransport");
  if (explicit) return explicit;
  return "https://localhost:8089/transport/wsta-control";
}

async function connectWstaTransportForever() {
  if (connecting) return;
  connecting = true;

  while (!wstaWriter) {
    try {
      if (!("WebTransport" in window)) {
        console.error("WebTransport is not available in this browser; retrying");
        await sleep(RETRY_MS);
        continue;
      }

      const url = defaultTransportUrl();
      console.log("wsta Makepad WebTransport connecting", url);

      wstaTransport = new WebTransport(url);
      await wstaTransport.ready;

      const stream = await wstaTransport.createBidirectionalStream();
      wstaWriter = stream.writable.getWriter();

      for (const line of pending) {
        await wstaWriter.write(new TextEncoder().encode(line + "\n"));
      }
      pending = [];

      readWstaTransport(stream.readable).catch((e) => {
        console.error("wsta read loop failed", e);
        resetAndRetry();
      });

      console.log("wsta Makepad WebTransport connected", url);
    } catch (e) {
      console.error("wsta Makepad WebTransport failed; retrying", e);
      wstaWriter = null;
      wstaTransport = null;
      await sleep(RETRY_MS);
    }
  }

  connecting = false;
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
      if (line) console.log("wsta backend -> makepad", line);
    }
  }

  resetAndRetry();
}

function resetAndRetry() {
  wstaWriter = null;
  wstaTransport = null;
  connecting = false;
  setTimeout(connectWstaTransportForever, RETRY_MS);
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export function wstaSendJson(json) {
  if (!wstaWriter) {
    pending.push(json);
    connectWstaTransportForever();
    return;
  }

  wstaWriter.write(new TextEncoder().encode(json + "\n")).catch((e) => {
    console.error("wsta send failed; queueing and retrying", e);
    pending.push(json);
    resetAndRetry();
  });
}

connectWstaTransportForever();
