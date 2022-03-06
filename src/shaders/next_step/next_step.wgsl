//For each enabled cell with coordinates (x, y) on prev_board we change the state of the neighbor cells on next_board.
//First, initialize next_board[x, y] = 0.
//If prev_board[x, y] == 1, then for each (xi, yj) in enabled_positions we calculate next_board[x + xi, y + yi] = (next_board[x + xi, y + yi] + 1) mod 2
//If prev_board[x, y] == 0, then for each (xi, yj) in enabled_positions we calculate next_board[x + xi, y + yi] = next_board[x + xi, y + yi]
//In other words, for each (xi, yj) in enabled_positions we calculate next_board[x + xi, y + yi] = (next_board[x + xi, y + yi] + prev_board[x, y]) mod 2
//Flipping this, we can calculate next_board[x, y] = SUM(prev_board[x - xi, y - yi]) mod 2 for all (xi, yi)
//After that, we calculate next_stability[x, y] as prev_stability[x, y] & (prev_board[x, y] == next_board[x, y])

//We store both boards and both stabilities as 2x2 quads. Each quad is packed into a single 32-bit value. The values prev_board[x, y] and next_board[x, y] refer to entire 2x2 quads.
//Each 2x2 quad is encoded as
// (Bits 0-7)   (Bits 8-15)
// (Bits 16-23) (Bits 24-31)
//Each workgroup has 8x8 threads, and each thread processes a single quad. The workgroup processes a 16x16 block of values.
//The largest click rule radius is 16, which means each workgroup might access eight more 16x16 blocks of values, surrounding the original one.
//These blocks are packed into quads too, so we need to store up to 24x24 quads.

//The original formula for next_board[x, y] can be extended to quads. Instead of using prev_board[x - xi, y - yi], we calculate the quad to add from the elements of prev_board.
//The formula to calculate this quad depends on the values of xi and yi:
//- If both xi and yi are divisible by 2, we add the quad at prev_board[xi / 2, yi / 2] to the quad at next_board[xi / 2, yi / 2].
//- If only yi is divisible by 2, we construct a new quad from the right half of prev_board[(xi - 1) / 2, yi / 2] and left half of prev_board[(xi + 1) / 2, yi / 2], and add it to next_board.
//- If only xi is divisible by 2, we construct a new quad from the top half of prev_board[xi / 2, (yi - 1) / 2] and bottom half of prev_board[xi / 2, (yi + 1) / 2], and add it to next_board.
//- If both xi and yi are indivisible by 2, we construct a new quad from the opposite values of four surrounding quads.

let click_rule_width  = 32u;
let click_rule_height = 32u;

let click_rule_data_width  = 16u; //click_rule_width  / 2
let click_rule_data_height = 16u; //click_rule_height / 2

let workgroup_threads_x = 8u;
let workgroup_threads_y = 8u;

struct ClickRuleData
{
    header_packed:            vec4<u32>;
    enabled_positions_packed: array<vec4<i32>, 512>; //click_rule_width * click_rule_height / 2
};

@group(0) @binding(0) var prev_board:     texture_2d<u32>;
@group(0) @binding(1) var prev_stability: texture_2d<u32>;

@group(0) @binding(2) var next_board:     texture_storage_2d<r32uint, write>;
@group(0) @binding(3) var next_stability: texture_storage_2d<r32uint, write>;

@group(0) @binding(4) var<uniform> click_rule_data: ClickRuleData;

var<workgroup> shared_quad_states: array<u32, 576>; //(click_rule_data_width + workgroup_threads_x) * (click_rule_data_height + workgroup_threads_y)

fn calculate_quad_index(local_thread_id: vec2<u32>, quad_offset: vec2<i32>, extra_radius_quads: u32) -> u32
{
    let quad_shared_state_width  = workgroup_threads_x + extra_radius_quads * 2u;
    let quad_shared_state_height = workgroup_threads_y + extra_radius_quads * 2u;

    let left_offset = u32(i32(extra_radius_quads) + quad_offset.x);
    let top_offset  = u32(i32(extra_radius_quads) + quad_offset.y);

    return (local_thread_id.y + top_offset) * quad_shared_state_width + (local_thread_id.x + left_offset);
}

fn calculate_quad(local_thread_id: vec2<u32>, click_rule_offset: vec2<i32>, extra_radius_quads: u32) -> u32
{
    let x_even: bool = (click_rule_offset.x % 2 == 0);
    let y_even: bool = (click_rule_offset.y % 2 == 0);

    if(x_even && y_even)
    {
        let quad_offset:       vec2<i32> = click_rule_offset / 2;
        let shared_quad_index: u32       = calculate_quad_index(local_thread_id, quad_offset, extra_radius_quads);

        return shared_quad_states[shared_quad_index];
    }
    else if(y_even)
    {
        let left_quad_mask:  u32 = 0x00ff00ffu;
        let right_quad_mask: u32 = 0xff00ff00u;

        let left_quad_offset  = vec2<i32>(click_rule_offset.x - 1, click_rule_offset.y) / 2;
        let right_quad_offset = vec2<i32>(click_rule_offset.x + 1, click_rule_offset.y) / 2;

        let left_shared_quad_index:  u32 = calculate_quad_index(local_thread_id, left_quad_offset,  extra_radius_quads);
        let right_shared_quad_index: u32 = calculate_quad_index(local_thread_id, right_quad_offset, extra_radius_quads);

        let right_half_left_quad: u32 = (shared_quad_states[left_shared_quad_index] & right_quad_mask);
        let left_half_right_quad: u32 = (shared_quad_states[right_shared_quad_index] & left_quad_mask);

        return (right_half_left_quad >> 8u) | (left_half_right_quad << 8u);
    }
    else if(x_even)
    {
        let top_quad_mask:    u32 = 0x0000ffffu;
        let bottom_quad_mask: u32 = 0xffff0000u;

        let top_quad_offset    = vec2<i32>(click_rule_offset.x, click_rule_offset.y - 1) / 2;
        let bottom_quad_offset = vec2<i32>(click_rule_offset.x, click_rule_offset.y + 1) / 2;

        let top_shared_quad_index:    u32 = calculate_quad_index(local_thread_id, top_quad_offset,    extra_radius_quads);
        let bottom_shared_quad_index: u32 = calculate_quad_index(local_thread_id, bottom_quad_offset, extra_radius_quads);

        let bottom_half_top_quad: u32 = (shared_quad_states[top_shared_quad_index] & bottom_quad_mask);
        let top_half_bottom_quad: u32 = (shared_quad_states[bottom_shared_quad_index] & top_quad_mask);

        return (bottom_half_top_quad >> 16u) | (top_half_bottom_quad << 16u);
    }
    else
    {
        let top_left_quad_mask:     u32 = 0x000000ffu;
        let top_right_quad_mask:    u32 = 0x0000ff00u;
        let bottom_left_quad_mask:  u32 = 0x00ff0000u;
        let bottom_right_quad_mask: u32 = 0xff000000u;

        let top_left_quad_offset     = vec2<i32>(click_rule_offset.x - 1, click_rule_offset.y - 1) / 2;
        let top_right_quad_offset    = vec2<i32>(click_rule_offset.x + 1, click_rule_offset.y - 1) / 2;
        let bottom_left_quad_offset  = vec2<i32>(click_rule_offset.x - 1, click_rule_offset.y + 1) / 2;
        let bottom_right_quad_offset = vec2<i32>(click_rule_offset.x + 1, click_rule_offset.y + 1) / 2;

        let top_left_shared_quad_index:     u32 = calculate_quad_index(local_thread_id, top_left_quad_offset,     extra_radius_quads);
        let top_right_shared_quad_index:    u32 = calculate_quad_index(local_thread_id, top_right_quad_offset,    extra_radius_quads);
        let bottom_left_shared_quad_index:  u32 = calculate_quad_index(local_thread_id, bottom_left_quad_offset,  extra_radius_quads);
        let bottom_right_shared_quad_index: u32 = calculate_quad_index(local_thread_id, bottom_right_quad_offset, extra_radius_quads);

        let bottom_right_of_top_left: u32 = (shared_quad_states[top_left_shared_quad_index] & bottom_right_quad_mask);
        let bottom_left_of_top_right: u32 = (shared_quad_states[top_right_shared_quad_index] & bottom_left_quad_mask);
        let top_right_of_bottom_left: u32 = (shared_quad_states[bottom_left_shared_quad_index] & top_right_quad_mask);
        let top_left_of_bottom_right: u32 = (shared_quad_states[bottom_right_shared_quad_index] & top_left_quad_mask);

        return (bottom_right_of_top_left >> 24u) | (bottom_left_of_top_right >> 8u) | (top_right_of_bottom_left << 8u) | (top_left_of_bottom_right << 24u);
    }
}

@stage(compute) @workgroup_size(8, 8)
fn main(@builtin(local_invocation_id) local_thread_id: vec3<u32>, @builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
    let element_count: u32 = click_rule_data.header_packed.x;
    let radius:        u32 = click_rule_data.header_packed.y;

    if(radius == 0u)
    {
        return;
    }

    let extra_radius:       u32 = radius - 1u;
    let extra_radius_quads: u32 = (extra_radius + 1u) / 2u;

    let quad_state_index: u32 = calculate_quad_index(local_thread_id.xy, vec2<i32>(0), extra_radius_quads);
    let prev_board_quad:  u32 = textureLoad(prev_board, vec2<i32>(global_thread_id.xy), 0).x;
    shared_quad_states[quad_state_index] = prev_board_quad;

    if(extra_radius_quads > 0u)
    {
        //Save eight extra 8x8 blocks into cache:
        //X X X
        //X o X
        //X X X
        //The threads saving the data for the block are the ones that, having the original block shifted to the one being saved,
        //have global_thread_id.xy still in bounds of extra_radius_quads
        let in_blocks = array<bool, 8>
        (
            (local_thread_id.x > workgroup_threads_x - extra_radius_quads) && (local_thread_id.y > workgroup_threads_y - extra_radius_quads),
                                                                              (local_thread_id.y > workgroup_threads_y - extra_radius_quads),
            (local_thread_id.x < extra_radius_quads)                       && (local_thread_id.y > workgroup_threads_y - extra_radius_quads),
            (local_thread_id.x > workgroup_threads_x - extra_radius_quads),
            (local_thread_id.x < extra_radius_quads),
            (local_thread_id.x > workgroup_threads_x - extra_radius_quads) && (local_thread_id.y < extra_radius_quads),
                                                                              (local_thread_id.y < extra_radius_quads),
            (local_thread_id.x < extra_radius_quads)                       && (local_thread_id.y < extra_radius_quads)
        );

        let block_offsets = array<vec2<i32>, 8>
        (
            vec2<i32>(-i32(workgroup_threads_x), -i32(workgroup_threads_y)),
            vec2<i32>( 0,                        -i32(workgroup_threads_y)),
            vec2<i32>( i32(workgroup_threads_x), -i32(workgroup_threads_y)),
            vec2<i32>(-i32(workgroup_threads_x),  0),
            vec2<i32>( i32(workgroup_threads_x),  0),
            vec2<i32>(-i32(workgroup_threads_x),  i32(workgroup_threads_y)),
            vec2<i32>( 0,                         i32(workgroup_threads_y)),
            vec2<i32>( i32(workgroup_threads_x),  i32(workgroup_threads_y))
        );

        //ThE eXpReSsIoN mAy OnLy Be InDeXeD bY a CoNsTaNt
        if(in_blocks[0])
        {
            let extra_quad_state_index = calculate_quad_index(local_thread_id.xy, block_offsets[0], extra_radius_quads);
            shared_quad_states[extra_quad_state_index] = textureLoad(prev_board, vec2<i32>(global_thread_id.xy) + block_offsets[0], 0).x;
        }

        if(in_blocks[1])
        {
            let extra_quad_state_index = calculate_quad_index(local_thread_id.xy, block_offsets[1], extra_radius_quads);
            shared_quad_states[extra_quad_state_index] = textureLoad(prev_board, vec2<i32>(global_thread_id.xy) + block_offsets[1], 0).x;
        }

        if(in_blocks[2])
        {
            let extra_quad_state_index = calculate_quad_index(local_thread_id.xy, block_offsets[2], extra_radius_quads);
            shared_quad_states[extra_quad_state_index] = textureLoad(prev_board, vec2<i32>(global_thread_id.xy) + block_offsets[2], 0).x;
        }

        if(in_blocks[3])
        {
            let extra_quad_state_index = calculate_quad_index(local_thread_id.xy, block_offsets[3], extra_radius_quads);
            shared_quad_states[extra_quad_state_index] = textureLoad(prev_board, vec2<i32>(global_thread_id.xy) + block_offsets[3], 0).x;
        }

        if(in_blocks[4])
        {
            let extra_quad_state_index = calculate_quad_index(local_thread_id.xy, block_offsets[4], extra_radius_quads);
            shared_quad_states[extra_quad_state_index] = textureLoad(prev_board, vec2<i32>(global_thread_id.xy) + block_offsets[4], 0).x;
        }

        if(in_blocks[5])
        {
            let extra_quad_state_index = calculate_quad_index(local_thread_id.xy, block_offsets[5], extra_radius_quads);
            shared_quad_states[extra_quad_state_index] = textureLoad(prev_board, vec2<i32>(global_thread_id.xy) + block_offsets[5], 0).x;
        }

        if(in_blocks[6])
        {
            let extra_quad_state_index = calculate_quad_index(local_thread_id.xy, block_offsets[6], extra_radius_quads);
            shared_quad_states[extra_quad_state_index] = textureLoad(prev_board, vec2<i32>(global_thread_id.xy) + block_offsets[6], 0).x;
        }

        if(in_blocks[7])
        {
            let extra_quad_state_index = calculate_quad_index(local_thread_id.xy, block_offsets[7], extra_radius_quads);
            shared_quad_states[extra_quad_state_index] = textureLoad(prev_board, vec2<i32>(global_thread_id.xy) + block_offsets[7], 0).x;
        }
    }

    workgroupBarrier();

    //Board size is always 2^n - 1. Mask out the bottom and the right edge
    let right_quad_mask:  u32 = 0xff00ff00u;
    let bottom_quad_mask: u32 = 0xffff0000u;

	let right_bottom_coordinates = vec2<u32>(textureDimensions(next_board) + vec2<i32>(-1, -1)); //Board size is always 2^n - 1. Mask out the bottom and right edge
	let on_right_bottom: vec2<bool> = (global_thread_id.xy == right_bottom_coordinates);

    let right_bottom_mask = vec2<u32>(right_quad_mask, bottom_quad_mask) * vec2<u32>(on_right_bottom);
    let modulo_2_mask     = 0x01010101u & ~(right_bottom_mask.x | right_bottom_mask.y);

    let packed_element_count = i32(element_count / 2u);

    var next_board_quad: u32 = 0x00000000u;
    for(var i: i32 = 0; i < packed_element_count; i = i + 1)
    {
        let offsets_packed: vec4<i32> = click_rule_data.enabled_positions_packed[i];

        let offset_1: vec2<i32> = offsets_packed.xy;
        let offset_2: vec2<i32> = offsets_packed.zw;

        let prev_board_quad_1: u32 = calculate_quad(local_thread_id.xy, offset_1, extra_radius_quads);
        let prev_board_quad_2: u32 = calculate_quad(local_thread_id.xy, offset_2, extra_radius_quads);

        next_board_quad = (next_board_quad + prev_board_quad_1 + prev_board_quad_2) & modulo_2_mask;
    }

    if(element_count % 2u != 0u)
    {
        let last_offset: vec2<i32> = click_rule_data.enabled_positions_packed[packed_element_count].xy;
        let prev_board_quad: u32 = calculate_quad(local_thread_id.xy, last_offset, extra_radius_quads);
        next_board_quad = (next_board_quad + prev_board_quad) & modulo_2_mask;
    }

    let prev_stability_quad: u32 = textureLoad(prev_stability, vec2<i32>(global_thread_id.xy), 0).x;
    let next_stability_quad: u32 = (prev_stability_quad & ~(prev_board_quad ^ next_board_quad));

    textureStore(next_board,     vec2<i32>(global_thread_id.xy), vec4<u32>(next_board_quad));
    textureStore(next_stability, vec2<i32>(global_thread_id.xy), vec4<u32>(next_stability_quad));
}