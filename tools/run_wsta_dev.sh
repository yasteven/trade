#!/usr/bin/env bash
set -euo pipefail

echo "== WSTA dev run: stable non-threaded Makepad WASM =="
./tools/build_wsta_makepad_wasm.sh
cargo run -p wsta
