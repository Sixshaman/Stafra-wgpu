if not exist static mkdir static

cd static

echo F|xcopy /Y /F "../src/index.html" "./index.html"

set RUSTFLAGS=--cfg=web_sys_unstable_apis

wasm-pack build --release --target web --out-name Stafra --out-dir static -- --features winit/web-sys

set RUSTFLAGS=
cd ..