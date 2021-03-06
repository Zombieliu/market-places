#!/bin/bash
set -e
rustup target add wasm32-unknown-unknown

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
mkdir -p res
cp target/wasm32-unknown-unknown/release/market.wasm ./res/market.wasm