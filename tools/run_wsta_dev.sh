#!/usr/bin/env bash
set -euo pipefail

echo "== WSTA dev run =="
echo "1) build/deploy Makepad WASM"
echo "2) run wsta host"

./tools/build_wsta_makepad_wasm.sh

echo
echo "== starting wsta =="
cargo run -p wsta
