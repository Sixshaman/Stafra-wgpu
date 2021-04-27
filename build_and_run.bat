if not exist static mkdir static

cd static

echo F|xcopy /Y /F "../src/index.html" "./index.html"

set RUSTFLAGS=--cfg=web_sys_unstable_apis

%VULKAN_SDK%/Bin/glslc ../src/shaders/clear_board/clear_4_corners.comp -o clear_4_corners.spv

%VULKAN_SDK%/Bin/glslc ../src/shaders/state_transform/initial_state_transform.comp -o initial_state_transform.spv
%VULKAN_SDK%/Bin/glslc ../src/shaders/state_transform/final_state_transform.comp   -o final_state_transform.spv

%VULKAN_SDK%/Bin/glslc ../src/shaders/render/render_state.vert -o render_state_vs.spv
%VULKAN_SDK%/Bin/glslc ../src/shaders/render/render_state.frag -o render_state_fs.spv

cargo build --release --no-default-features --target wasm32-unknown-unknown --features winit/web-sys
wasm-bindgen --no-typescript --target web --out-name Stafra --out-dir ../static ../target/wasm32-unknown-unknown/release/stafra_wgpu.wasm
miniserve --index index.html

set RUSTFLAGS=
cd ..