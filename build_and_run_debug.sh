export CC=gcc
export CXX=g++

mkdir -p static
mkdir -p target
mkdir -p target/shaders

# shellcheck disable=SC2164
cd static

cp ../src/stafra.html stafra.html
cp ../src/stafra.css stafra.css

cp ../src/shaders/clear_board/clear_4_corners.wgsl ../target/shaders/clear_4_corners.wgsl
cp ../src/shaders/clear_board/clear_4_sides.wgsl   ../target/shaders/clear_4_sides.wgsl
cp ../src/shaders/clear_board/clear_center.wgsl    ../target/shaders/clear_center.wgsl

cp ../src/shaders/next_step/next_step.wgsl ../target/shaders/next_step.wgsl

cp ../src/shaders/click_rule/bake_click_rule.wgsl ../target/shaders/bake_click_rule.wgsl

cp ../src/shaders/state_transform/initial_state_transform.wgsl ../target/shaders/initial_state_transform.wgsl
cp ../src/shaders/state_transform/final_state_transform.wgsl   ../target/shaders/final_state_transform.wgsl
cp ../src/shaders/state_transform/clear_stability.wgsl         ../target/shaders/clear_stability.wgsl

cp ../src/shaders/render/render_state_vs.wgsl ../target/shaders/render_state_vs.wgsl
cp ../src/shaders/render/render_state_fs.wgsl ../target/shaders/render_state_fs.wgsl

cp ../src/shaders/mip/final_state_generate_next_mip.wgsl ../target/shaders/final_state_generate_next_mip.wgsl

RUSTFLAGS=--cfg=web_sys_unstable_apis wasm-pack build --release --target web --out-name Stafra --out-dir static
miniserve --index stafra.html