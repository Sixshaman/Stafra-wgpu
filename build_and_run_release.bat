if not exist static mkdir static
if not exist target mkdir target

cd target
if not exist shaders mkdir shaders
cd ..

cd static

echo F|xcopy /Y /F "../src/stafra.html" "./stafra.html"
echo F|xcopy /Y /F "../src/stafra.css" "./stafra.css"

set RUSTFLAGS=--cfg=web_sys_unstable_apis

%VULKAN_SDK%/Bin/glslc ../src/shaders/clear_board/clear_4_corners.comp -o ../target/shaders/clear_4_corners.spv

%VULKAN_SDK%/Bin/glslc ../src/shaders/next_step/next_step.comp -o ../target/shaders/next_step.spv

%VULKAN_SDK%/Bin/glslc ../src/shaders/state_transform/initial_state_transform.comp -o ../target/shaders/initial_state_transform.spv
%VULKAN_SDK%/Bin/glslc ../src/shaders/state_transform/final_state_transform.comp   -o ../target/shaders/final_state_transform.spv
%VULKAN_SDK%/Bin/glslc ../src/shaders/state_transform/clear_stability.comp         -o ../target/shaders/clear_stability.spv

%VULKAN_SDK%/Bin/glslc ../src/shaders/render/render_state.vert -o ../target/shaders/render_state_vs.spv
%VULKAN_SDK%/Bin/glslc ../src/shaders/render/render_state.frag -o ../target/shaders/render_state_fs.spv

%VULKAN_SDK%/Bin/glslc ../src/shaders/mip/final_state_generate_next_mip.comp -o ../target/shaders/final_state_generate_next_mip.spv

wasm-pack build --release --target web --out-name Stafra --out-dir static -- --features winit/web-sys
miniserve --index stafra.html

set RUSTFLAGS=
cd ..