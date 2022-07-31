//Generates the next mip level for a final image

@group(0) @binding(0) var in_mip:  texture_2d<f32>;
@group(0) @binding(1) var out_mip: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
	let quad_start = vec2<i32>(global_thread_id.xy) * 2;

    let top_left:     vec4<f32> = textureLoad(in_mip, quad_start + vec2<i32>(0, 0), 0);
    let top_right:    vec4<f32> = textureLoad(in_mip, quad_start + vec2<i32>(1, 0), 0);
    let bottom_left:  vec4<f32> = textureLoad(in_mip, quad_start + vec2<i32>(0, 1), 0);
    let bottom_right: vec4<f32> = textureLoad(in_mip, quad_start + vec2<i32>(1, 1), 0);

    let downsampled: vec4<f32> = (top_left + top_right + bottom_left + bottom_right) * 0.25;
    textureStore(out_mip, vec2<i32>(global_thread_id.xy), downsampled);
}
