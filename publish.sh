#!/usr/bin/env sh
echo CLEANING UP
rm -rf ./web_dist
rm dist.zip

echo BUILDING...
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --out-dir ./web_dist/ --target web ./target/wasm32-unknown-unknown/release/prashan.wasm

echo PACKAGING...
7z a dist.zip index.html assets web_dist

echo PUBLISHING...
butler push dist.zip septumca/prashan:web