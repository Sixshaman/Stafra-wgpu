if not exist static mkdir static

cd static

set RUSTFLAGS=--cfg=web_sys_unstable_apis

wasm-pack build --release --target web --out-name Stafra --out-dir static

set RUSTFLAGS=
cd ..