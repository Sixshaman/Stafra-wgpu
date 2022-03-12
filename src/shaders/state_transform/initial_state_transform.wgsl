//Transfroms the initial state texture into the stability buffer data

@group(0) @binding(0) var initial_state_tex: texture_2d<f32>;
@group(0) @binding(1) var out_initial_board: texture_storage_2d<r32uint, write>;

@stage(compute) @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
    let initial_state_size: vec2<i32> = textureDimensions(initial_state_tex);
    let board_size:         vec2<i32> = textureDimensions(out_initial_board);

    let real_board_size: vec2<i32> = board_size * 2 - vec2<i32>(1, 1);
    let offset: vec2<i32> = (initial_state_size - real_board_size) / 2;

	let lum_factor = vec4<f32>(0.2126, 0.7152, 0.0722, 0.0);
	let quad_start = vec2<i32>(global_thread_id.xy) * 2 + offset;

	//4 packed values
    let top_left_quad_coord:     vec2<i32> = quad_start + vec2<i32>(0, 0);
    let top_right_quad_coord:    vec2<i32> = quad_start + vec2<i32>(1, 0);
    let bottom_left_quad_coord:  vec2<i32> = quad_start + vec2<i32>(0, 1);
    let bottom_right_quad_coord: vec2<i32> = quad_start + vec2<i32>(1, 1);

    let top_left_quad_in_bounds:     bool = all(top_left_quad_coord >= vec2(0, 0))     && all(top_left_quad_coord < initial_state_size);
    let top_right_quad_in_bounds:    bool = all(top_right_quad_coord >= vec2(0, 0))    && all(top_right_quad_coord < initial_state_size);
    let bottom_left_quad_in_bounds:  bool = all(bottom_left_quad_coord >= vec2(0, 0))  && all(bottom_left_quad_coord < initial_state_size);
    let bottom_right_quad_in_bounds: bool = all(bottom_right_quad_coord >= vec2(0, 0)) && all(bottom_right_quad_coord < initial_state_size);

	let top_left_quad:     vec4<f32> = textureLoad(initial_state_tex, top_left_quad_coord,     0) * f32(top_left_quad_in_bounds);
	let top_right_quad:    vec4<f32> = textureLoad(initial_state_tex, top_right_quad_coord,    0) * f32(top_right_quad_in_bounds);
	let bottom_left_quad:  vec4<f32> = textureLoad(initial_state_tex, bottom_left_quad_coord,  0) * f32(bottom_left_quad_in_bounds);
	let bottom_right_quad: vec4<f32> = textureLoad(initial_state_tex, bottom_right_quad_coord, 0) * f32(bottom_right_quad_in_bounds);

	let state_color_matrix = mat4x4<f32>(top_left_quad, top_right_quad, bottom_left_quad, bottom_right_quad);
	let quad_states        = vec4<u32>(lum_factor * state_color_matrix > vec4<f32>(0.15));

	//Pack a uint32-encoded quad   from a bvec4-encoded quad:
	// (Bits 0-7)   (Bits 8-15)    (values.x) (values.y)
	// (Bits 16-23) (Bits 24-31)   (values.z) (values.w)
	let packed_quad: u32 = ((quad_states.x & 0xffu) <<  0u) | ((quad_states.y & 0xffu) <<  8u)
	                     | ((quad_states.z & 0xffu) << 16u) | ((quad_states.w & 0xffu) << 24u);

	//Masks for halves of 2x2 quad, encoded as
	// (Bits 0-7)   (Bits 8-15)
	// (Bits 16-23) (Bits 24-31)
	let right_quad_mask:  u32 = 0xff00ff00u;
	let bottom_quad_mask: u32 = 0xffff0000u;

	let right_bottom_coordinates = vec2<u32>(textureDimensions(out_initial_board) + vec2<i32>(-1, -1)); //Board size is always 2^n - 1. Mask out the bottom and right edge
	let on_right_bottom: vec2<bool> = (global_thread_id.xy == right_bottom_coordinates);

	let right_bottom_mask = vec2<u32>(right_quad_mask, bottom_quad_mask) * vec2<u32>(on_right_bottom);
    textureStore(out_initial_board, vec2<i32>(global_thread_id.xy), vec4<u32>(packed_quad & ~(right_bottom_mask.x | right_bottom_mask.y)));
}