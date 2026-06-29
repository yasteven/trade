# wsta_makepad

Final Makepad/WASM frontend for `trade/wsta`.

This replaces the temporary HTML/JS control scaffold.

Boundaries:

```text
wsta_makepad
  Makepad Rust UI
  compiles to browser WASM
  emits BrowserToWsta packets

wsta_makepad/resources/web/wsta_transport.js
  browser WebTransport API adapter only
  not UI

wsta
  axum host
  serves Makepad WASM artifacts
  owns actor/select loop
  owns dsta seek bridge
```

Expected generated browser artifacts location:

```text
wsta_makepad/target/makepad-wasm/
```

Expected served URLs:

```text
http://127.0.0.1:8088/
http://127.0.0.1:8088/makepad/
```

Expected Makepad loader assets:

```text
/makepad/wsta_makepad.js
/makepad/wsta_makepad_bg.wasm
/makepad/wsta_transport.js
```

No HTML controls should be added to `wsta/src/web/embedded_index_html.rs`.
That file is only the Makepad loader.
