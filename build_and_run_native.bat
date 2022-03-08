if not exist target mkdir target

cd target
if not exist shaders mkdir shaders
cd ..

cd static

set RUSTFLAGS=--cfg=web_sys_unstable_apis
set RUST_LOG=warn

echo F|xcopy /Y /F "../src/shaders/render/render_state_vs.wgsl"            "../target/shaders/render_state_vs.wgsl"
echo F|xcopy /Y /F "../src/shaders/render/render_state_fs.wgsl"            "../target/shaders/render_state_fs.wgsl"
echo F|xcopy /Y /F "../src/shaders/render/click_rule_render_state_vs.wgsl" "../target/shaders/click_rule_render_state_vs.wgsl"
echo F|xcopy /Y /F "../src/shaders/render/click_rule_render_state_fs.wgsl" "../target/shaders/click_rule_render_state_fs.wgsl"

echo F|xcopy /Y /F "../src/shaders/clear_board/clear_4_corners.wgsl" "../target/shaders/clear_4_corners.wgsl"
echo F|xcopy /Y /F "../src/shaders/clear_board/clear_4_sides.wgsl"   "../target/shaders/clear_4_sides.wgsl"
echo F|xcopy /Y /F "../src/shaders/clear_board/clear_center.wgsl"    "../target/shaders/clear_center.wgsl"

echo F|xcopy /Y /F "../src/shaders/state_transform/initial_state_transform.wgsl" "../target/shaders/initial_state_transform.wgsl"
echo F|xcopy /Y /F "../src/shaders/state_transform/final_state_transform.wgsl"   "../target/shaders/final_state_transform.wgsl"
echo F|xcopy /Y /F "../src/shaders/state_transform/clear_stability.wgsl"         "../target/shaders/clear_stability.wgsl"

echo F|xcopy /Y /F "../src/shaders/mip/final_state_generate_next_mip.wgsl" "../target/shaders/final_state_generate_next_mip.wgsl"

echo F|xcopy /Y /F "../src/shaders/next_step/next_step.wgsl" "../target/shaders/next_step.wgsl"

echo F|xcopy /Y /F "../src/shaders/click_rule/bake_click_rule.wgsl" "../target/shaders/bake_click_rule.wgsl"

cargo run

set RUSTFLAGS=
set RUST_LOG=
cd ..