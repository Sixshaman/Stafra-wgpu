//Clears the restriction texture

@group(0) @binding(0) var out_restriction: texture_storage_2d<r32uint, write>;

@stage(compute) @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
    //Init every quad to "no restriction" (0xff in each of 4 bytes)
    let packed_restriction: u32 = 0xffffffffu;

    //Masks for halves of a 2x2 quad
    let right_quad_mask:  u32 = 0xff00ff00u;
    let bottom_quad_mask: u32 = 0xffff0000u;

    //Mask out the rightmost and the bottommost values
	let right_bottom_coordinates = vec2<u32>(textureDimensions(out_restriction) + vec2<i32>(-1, -1));
	let on_right_bottom: vec2<bool> = (global_thread_id.xy == right_bottom_coordinates);

	let right_bottom_mask = vec2<u32>(right_quad_mask, bottom_quad_mask) * vec2<u32>(on_right_bottom);
    textureStore(out_restriction, vec2<i32>(global_thread_id.xy), vec4<u32>(packed_restriction & ~(right_bottom_mask.x | right_bottom_mask.y)));
}