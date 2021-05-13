export CC=gcc
export CXX=g++

mkdir -p static
mkdir -p target
mkdir -p target/shaders

# shellcheck disable=SC2164
cd static

cp ../src/stafra.html stafra.html
cp ../src/stafra.css stafra.css

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/clear_board/clear_4_corners.comp -O0 -o ../target/shaders/clear_4_corners.spv

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/next_step/next_step.comp -O0 -o ../target/shaders/next_step.spv

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/state_transform/initial_state_transform.comp -O0 -o ../target/shaders/initial_state_transform.spv
${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/state_transform/final_state_transform.comp   -O0 -o ../target/shaders/final_state_transform.spv
${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/state_transform/clear_stability.comp         -O0 -o ../target/shaders/clear_stability.spv

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/render/render_state.vert -O0 -o ../target/shaders/render_state_vs.spv
${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/render/render_state.frag -O0 -o ../target/shaders/render_state_fs.spv

${VULKAN_SDK}/x86_64/bin/glslc ../src/shaders/mip/final_state_generate_next_mip.comp -O0 -o ../target/shaders/final_state_generate_next_mip.spv

RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --release --target web --out-name Stafra --out-dir static -- --features winit/web-sys
miniserve --index stafra.html