export CC=gcc
export CXX=g++

cd static

${VULKAN_SDK}/x86_64/bin/glslc ../src/main.vert -o main_vs.spv 
${VULKAN_SDK}/x86_64/bin/glslc ../src/main.frag -o main_fs.spv 

RUSTFLAGS=--cfg=web_sys_unstable_apis cargo build --no-default-features --target wasm32-unknown-unknown --features winit/web-sys
wasm-bindgen --no-typescript --target web --out-name Stafra --out-dir ../static ../target/wasm32-unknown-unknown/debug/stafra_wgpu.wasm
miniserve --index index.html