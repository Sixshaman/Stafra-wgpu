export CC=gcc
export CXX=g++

mkdir -p static

cd static

cp ../src/index.html index.html

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/clear_board/clear_4_corners.comp -o clear_4_corners.spv

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/next_step/next_step.comp -o next_step.spv

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/state_transform/initial_state_transform.comp -o initial_state_transform.spv
${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/state_transform/final_state_transform.comp   -o final_state_transform.spv

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/render/render_state.vert -o render_state_vs.spv
${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/render/render_state.frag -o render_state_fs.spv

RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --release --target web --out-name Stafra --out-dir static -- --features winit/web-sys
miniserve --index index.html