export CC=gcc
export CXX=g++

mkdir -p static
mkdir -p target

cp ./src/stafra.html ./static/stafra.html
cp ./src/stafra.css  ./static/stafra.css

RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --lib --debug --target wasm32-unknown-unknown
wasm-bindgen --target web --out-name stafra --out-dir static ./target/wasm32-unknown-unknown/release/stafra.wasm

# shellcheck disable=SC2164
cd static
miniserve --index stafra.html