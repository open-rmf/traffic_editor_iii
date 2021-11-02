#!/bin/bash
set -o verbose
set -o errexit
export CARGO_PROFILE_RELEASE_LTO=true
export CARGO_PROFILE_RELEASE_OPT_LEVEL=z
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir web target/wasm32-unknown-unknown/release/libtraffic_editor_iii.wasm
cd web
wasm-opt -Oz -o optimized_for_size.wasm libtraffic_editor_iii_bg.wasm
mv optimized_for_size.wasm libtraffic_editor_iii_bg.wasm
