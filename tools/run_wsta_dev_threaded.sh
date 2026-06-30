#!/usr/bin/env bash
set -euo pipefail

echo "== WSTA dev run: threaded Makepad WASM =="
./tools/build_wsta_makepad_wasm_threaded.sh
cargo run -p wsta
