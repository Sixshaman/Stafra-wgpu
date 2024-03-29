//Clears the stability texture

@group(0) @binding(0) var out_initial_stability: texture_storage_2d<r32uint, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
    let board_size: vec2<i32> = textureDimensions(out_initial_stability);

	if(global_thread_id.x >= u32(board_size.x) || global_thread_id.y >= u32(board_size.y))
	{
	    return;
	}

    //Init every quad to "stable" (0 in each of 4 bytes)
    let packed_quad = 0u;

    //Masks for halves of a 2x2 quad
    let right_quad_mask:  u32 = 0xff00ff00u;
    let bottom_quad_mask: u32 = 0xffff0000u;

	let right_bottom_coordinates = vec2<u32>(board_size + vec2<i32>(-1, -1)); //Board size is always 2^n - 1. Mask out the bottom and right edge
	let on_right_bottom: vec2<bool> = (global_thread_id.xy == right_bottom_coordinates);

	let right_bottom_mask = vec2<u32>(right_quad_mask, bottom_quad_mask) * vec2<u32>(on_right_bottom);
    textureStore(out_initial_stability, vec2<i32>(global_thread_id.xy), vec4<u32>(packed_quad | (right_bottom_mask.x | right_bottom_mask.y)));
}