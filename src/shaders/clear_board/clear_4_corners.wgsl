@group(0) @binding(0) var out_initial_board: texture_storage_2d<r32uint, write>;

@stage(compute) @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
	let board_size: vec2<i32> = textureDimensions(out_initial_board);

	let top_left     = vec2<u32>(0u,                    0u);
	let top_right    = vec2<u32>(u32(board_size.x - 1), 0u);
	let bottom_left  = vec2<u32>(0u,                    u32(board_size.y - 1));
	let bottom_right = vec2<u32>(u32(board_size.x - 1), u32(board_size.y - 1));

	//Each thread processes a single 2x2 quad
	let in_corner_quad_values = vec4<bool>(all(global_thread_id.xy == top_left),    all(global_thread_id.xy == top_right),
	                                       all(global_thread_id.xy == bottom_left), all(global_thread_id.xy == bottom_right));

	//Pack a uint32-encoded quad:
	// (Bits 0-7)   (Bits 8-15)
	// (Bits 16-23) (Bits 24-31)
	//The expression vec4<u32>(in_corner) operation creates a quad with a possible top-left flag.
	//It creates a correct value for all corners, because the board size is 2^n - 1, and each 2x2 quad in a corner corresponds only to the top-left value of encoded quad
	let in_corner = u32(any(in_corner_quad_values));
    textureStore(out_initial_board, vec2<i32>(global_thread_id.xy), vec4<u32>(in_corner));
}