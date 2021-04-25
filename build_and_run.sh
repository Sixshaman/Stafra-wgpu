export CC=gcc
export CXX=g++

cd static

RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --release --no-default-features --target wasm32-unknown-unknown --features winit/web-sys
wasm-bindgen --no-typescript --target web --out-name Stafra --out-dir ../static ../target/wasm32-unknown-unknown/release/stafra_wgpu.wasm
miniserve --index index.html