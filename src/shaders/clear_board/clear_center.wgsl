@group(0) @binding(0) var out_initial_board: texture_storage_2d<r32uint, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
    let board_size: vec2<i32> = textureDimensions(out_initial_board);

	if(global_thread_id.x >= u32(board_size.x) || global_thread_id.y >= u32(board_size.y))
	{
	    return;
	}

    let center        = vec2<u32>(u32(board_size.x - 1) / 2u, u32(board_size.y - 1) / 2u);
    let inside_center = u32(all(global_thread_id.xy == center));

    //Each thread processes a single 2x2 quad. The quad is packed into uint32:
    // (Bits 0-7)   (Bits 8-15)
    // (Bits 16-23) (Bits 24-31)
    //For >1x1 boards the center corresponds to the bottom-right flag of the corresponding center quad
    //For 1x1 boards the center corresponds to the top-left flag of the corresponding center quad
    let is_1x1           = u32(board_size.x == 1 && board_size.y == 1);
    let is_not_1x1       = u32(!bool(is_1x1));
    let center_bit_state = is_not_1x1 * inside_center;
    let in_center  = (is_1x1 << 0u)  | (0u               << 8u)
                   | (0u     << 16u) | (center_bit_state << 24u);

    textureStore(out_initial_board, vec2<i32>(global_thread_id.xy), vec4<u32>(in_center));
}