# wsta

`wsta` is the browser-hosted replacement for the old Iced `vsta` frontend.

Final boundaries:

- `server`: axum host for final browser / Makepad page assets.
- `actor`: core_front_main-style tokio select loop.
- `seek_bridge`: existing `dsta::ksta` mesh bridge.
- `web`: embedded browser shell.

Rules:

- No HTTP control fallback.
- Browser controls use WebTransport.
- Bot protocol stays in `dsta`.
- `Snively` goes browser -> wsta actor -> seek bridge -> backend.
- `MadeBot`, `GotTtai`, `GotTick00`, and `GotTick01` go backend -> seek bridge -> wsta actor -> browser.

Default env:

- `WSTA_HTTP_BIND=0.0.0.0:8088`
- `WSTA_SEEK_BACKEND_ADDR=127.0.0.1:4433`

Browser page:

- `http://127.0.0.1:8088/`
- `http://127.0.0.1:8088/makepad/`

Browser WebTransport endpoint expected by page:

- `https://localhost:8089/transport/wsta-control`

The WebTransport control endpoint is intentionally separate from the HTTP asset host, matching the SOLS split.
