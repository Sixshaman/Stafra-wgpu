if not exist static mkdir static

cd static

echo F|xcopy /Y /F "../src/index.html" "./index.html"

set RUSTFLAGS=--cfg=web_sys_unstable_apis

cargo build --release --no-default-features --target wasm32-unknown-unknown --features winit/web-sys
wasm-bindgen --no-typescript --target web --out-name Stafra --out-dir ../static ../target/wasm32-unknown-unknown/release/stafra_wgpu.wasm

set RUSTFLAGS=
cd ..