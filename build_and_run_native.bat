if not exist target mkdir target

cd target
if not exist shaders mkdir shaders
cd ..

cd static

set RUSTFLAGS=--cfg=web_sys_unstable_apis
set RUST_LOG=warn

%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/clear_4_corners.spv ../src/shaders/clear_board/clear_4_corners.comp
%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/clear_4_sides.spv   ../src/shaders/clear_board/clear_4_sides.comp
%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/clear_center.spv    ../src/shaders/clear_board/clear_center.comp

%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/next_step.spv ../src/shaders/next_step/next_step.comp

%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/initial_state_transform.spv ../src/shaders/state_transform/initial_state_transform.comp
%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/final_state_transform.spv   ../src/shaders/state_transform/final_state_transform.comp
%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/clear_stability.spv         ../src/shaders/state_transform/clear_stability.comp

%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/render_state_vs.spv ../src/shaders/render/render_state.vert
%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/render_state_fs.spv ../src/shaders/render/render_state.frag

%VULKAN_SDK%/Bin/glslangValidator -V --target-env spirv1.3 -g -o ../target/shaders/final_state_generate_next_mip.spv ../src/shaders/mip/final_state_generate_next_mip.comp

cargo run

set RUSTFLAGS=
set RUST_LOG=
cd ..