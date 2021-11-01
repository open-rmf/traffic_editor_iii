#!/bin/bash
set -o verbose
set -o errexit
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web target/wasm32-unknown-unknown/release/libtraffic_editor_iii.wasm
