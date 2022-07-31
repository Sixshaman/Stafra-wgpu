@group(0) @binding(0) var out_initial_board: texture_storage_2d<r32uint, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
	let board_size: vec2<i32> = textureDimensions(out_initial_board);

	if(global_thread_id.x >= u32(board_size.x) || global_thread_id.y >= u32(board_size.y))
	{
	    return;
	}

    let center_left   = vec2<u32>(0u,                          u32(board_size.y - 1) / 2u);
    let center_top    = vec2<u32>(u32(board_size.x - 1) / 2u,  0u);
    let center_right  = vec2<u32>(u32(board_size.x - 1),       u32(board_size.y - 1) / 2u);
    let center_bottom = vec2<u32>(u32(board_size.x - 1) / 2u,  u32(board_size.y - 1));

    let inside_horizontal = u32(all(global_thread_id.xy == center_left) || all(global_thread_id.xy == center_right));
    let inside_vertical   = u32(all(global_thread_id.xy == center_top)  || all(global_thread_id.xy == center_bottom));

    //Each thread processes a single 2x2 quad. The quad is packed into uint32:
    // (Bits 0-7)   (Bits 8-15)
    // (Bits 16-23) (Bits 24-31)
    //Top and bottom centers correspond to the top-right flag of the corresponding center quads
    //Left and right centers correspond to the bottom-left flag of the corresponding center quads
    //1x1 boards are exception and their centers correspond to the top-left flag
    let is_1x1 = u32(board_size.x == 1 && board_size.y == 1);
    let in_side_center_quad_values = (is_1x1            << 0u)  | (inside_vertical << 8u)
                                   | (inside_horizontal << 16u) | (0u              << 24u);

    textureStore(out_initial_board, vec2<i32>(global_thread_id.xy), vec4<u32>(in_side_center_quad_values));
}