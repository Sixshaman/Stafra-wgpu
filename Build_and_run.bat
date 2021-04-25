cd static
cargo build --release --target wasm32-unknown-unknown --features winit/web-sys
wasm-bindgen --no-typescript --target web --out-name Stafra --out-dir ..\static ..\target\wasm32-unknown-unknown\release\Stafra_wgpu.wasm
miniserve --index index.html