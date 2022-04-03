//Transfroms the stability buffer data into a colored image
//Mip generation is handled by another shader

struct SpawnData
{
    spawn_period:     u32,
    smooth_transform: u32
};

@group(0) @binding(0) var          final_board:   texture_2d<u32>;
@group(0) @binding(1) var          out_tex_mip_0: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> spawn_data:    SpawnData;

@stage(compute) @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
	let final_stability_encoded: u32 = textureLoad(final_board, vec2<i32>(global_thread_id.xy), 0).x;

	//Unpack the encoded quad
	var final_stability_quad = vec4<u32>((final_stability_encoded >>  0u) & 0xffu, (final_stability_encoded >>  8u) & 0xffu,
	                                     (final_stability_encoded >> 16u) & 0xffu, (final_stability_encoded >> 24u) & 0xffu);
	                                     
    //0 -> spawn period, 1 -> 0, 2 -> 1, ...
    final_stability_quad = clamp(final_stability_quad - vec4<u32>(1u), vec4<u32>(0u), vec4<u32>(spawn_data.spawn_period));
    if(spawn_data.smooth_transform == 0u)
    {
        let tex_value = vec4<f32>(final_stability_quad == vec4<u32>(spawn_data.spawn_period));
        textureStore(out_tex_mip_0, vec2<i32>(global_thread_id.xy), tex_value);
	}
	else
	{
	    let tex_value = vec4<f32>(final_stability_quad) / vec4<f32>(f32(spawn_data.spawn_period));
	    textureStore(out_tex_mip_0, vec2<i32>(global_thread_id.xy), tex_value);
	}
}