//Transfroms the stability buffer data into a colored image
//Mip generation is handled by another shader

@group(0) @binding(0) var final_board:   texture_2d<u32>;
@group(0) @binding(1) var out_tex_mip_0: texture_storage_2d<rgba8unorm, write>;

@stage(compute) @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
	let final_stability_encoded: u32 = textureLoad(final_board, vec2<i32>(global_thread_id.xy), 0).x;

	//Unpack the encoded quad
	var final_stability_quad = vec4<u32>((final_stability_encoded >>  0u) & 0xffu, (final_stability_encoded >>  8u) & 0xffu,
	                                     (final_stability_encoded >> 16u) & 0xffu, (final_stability_encoded >> 24u) & 0xffu);

	final_stability_quad = final_stability_quad * vec4<u32>(final_stability_quad < vec4<u32>(2u)); //Zero out all values that have final stability >= 2

	let tex_value = vec4<f32>(final_stability_quad);
	textureStore(out_tex_mip_0, vec2<i32>(global_thread_id.xy), tex_value);
}