// trade/wsta_makepad/resources/web/wsta_transport.js
//
// Browser adapter boundary for wsta_makepad.
// This file must never prevent the Makepad UI from rendering.
// It queues JSON packets until a real WebTransport session exists.

const wstaState = globalThis.__wstaTransportState ?? {
  queue: [],
  connected: false,
  lastError: "not connected",
  session: null,
  writer: null,
};

globalThis.__wstaTransportState = wstaState;

function setError(err) {
  wstaState.connected = false;
  wstaState.lastError = err && err.message ? err.message : String(err);
  console.warn("[wsta_transport]", wstaState.lastError);
}

async function connectWebTransport() {
  if (!("WebTransport" in globalThis)) {
    setError("WebTransport is not available in this browser/context");
    return;
  }

  // Final target will be the WSTA WebTransport endpoint.
  // This stub intentionally does not block UI startup if that endpoint is absent.
  const url = globalThis.WSTA_WEBTRANSPORT_URL || null;
  if (!url) {
    setError("WSTA_WEBTRANSPORT_URL not set; packets are queued");
    return;
  }

  try {
    const transport = new WebTransport(url);
    await transport.ready;

    const stream = await transport.createUnidirectionalStream();
    const writer = stream.getWriter();

    wstaState.session = transport;
    wstaState.writer = writer;
    wstaState.connected = true;
    wstaState.lastError = "connected";

    while (wstaState.queue.length > 0) {
      const json = wstaState.queue.shift();
      await writer.write(new TextEncoder().encode(json + "\n"));
    }

    transport.closed.catch(setError);
  } catch (e) {
    setError(e);
  }
}

export function wstaSendJson(json) {
  try {
    if (wstaState.connected && wstaState.writer) {
      wstaState.writer.write(new TextEncoder().encode(String(json) + "\n")).catch(setError);
    } else {
      wstaState.queue.push(String(json));
      if (wstaState.queue.length > 512) {
        wstaState.queue.splice(0, wstaState.queue.length - 512);
      }
    }

    console.log("[wsta_transport queued/sent]", json);
    return true;
  } catch (e) {
    setError(e);
    return false;
  }
}

export function wstaTransportStatus() {
  return {
    connected: wstaState.connected,
    queued: wstaState.queue.length,
    lastError: wstaState.lastError,
  };
}

// Start asynchronously and never throw during module import.
connectWebTransport().catch(setError);
