#!/bin/bash
set -eu

# Adapted from https://github.com/creativcoder/headlines/blob/main/setup_web.sh

cargo build --release -p minesweep-rs --lib --target wasm32-unknown-unknown

wasm-bindgen target/wasm32-unknown-unknown/release/minesweep_rs.wasm --out-dir webapp --no-modules --no-typescript

cd webapp
basic-http-server --addr 127.0.0.1:3000 .
