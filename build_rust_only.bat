if not exist static mkdir static

set RUSTFLAGS=--cfg=web_sys_unstable_apis

wasm-pack build --release --target web --out-name Stafra --out-dir static

del static/.gitignore
del static/README.md

set RUSTFLAGS=
cd ..