#!/usr/bin/env bash
set -euo pipefail

root_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root_dir"

cargo build --offline --features debug-hook --target wasm32-unknown-unknown
wasm-bindgen \
  --target web \
  --debug \
  --keep-debug \
  --out-dir web/pkg \
  target/wasm32-unknown-unknown/debug/wokwi_elements_rs.wasm
