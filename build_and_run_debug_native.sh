export CC=gcc
export CXX=g++

mkdir -p static
mkdir -p target
mkdir -p target/shaders

# shellcheck disable=SC2164
cd static

cp ../src/stafra.html stafra.html
cp ../src/stafra.css stafra.css

${VULKAN_SDK}/x86_64/bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/clear_4_corners.spv ../src/shaders/clear_board/clear_4_corners.comp

${VULKAN_SDK}/x86_64/bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/next_step.spv ../src/shaders/next_step/next_step.comp

${VULKAN_SDK}/x86_64/bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/initial_state_transform.spv ../src/shaders/state_transform/initial_state_transform.comp
${VULKAN_SDK}/x86_64/bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/final_state_transform.spv   ../src/shaders/state_transform/final_state_transform.comp
${VULKAN_SDK}/x86_64/bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/clear_stability.spv         ../src/shaders/state_transform/clear_stability.comp

${VULKAN_SDK}/x86_64/bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/render_state_vs.spv ../src/shaders/render/render_state.vert
${VULKAN_SDK}/x86_64/bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/render_state_fs.spv ../src/shaders/render/render_state.frag

${VULKAN_SDK}/x86_64/bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/final_state_generate_next_mip.spv ../src/shaders/mip/final_state_generate_next_mip.comp

cargo run