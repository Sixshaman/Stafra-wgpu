if not exist static mkdir static
if not exist target mkdir target

set RUSTFLAGS=--cfg=web_sys_unstable_apis

echo F|xcopy /Y /F "./src/stafra.html" "./static/stafra.html"
echo F|xcopy /Y /F "./src/stafra.css"  "./static/stafra.css"

cargo build --lib --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-name stafra --out-dir static ./target/wasm32-unknown-unknown/release/stafra.wasm

cd static
del .gitignore
del README.md

miniserve --index stafra.html

set RUSTFLAGS=
cd ..