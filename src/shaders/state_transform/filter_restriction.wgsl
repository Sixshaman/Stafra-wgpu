//Applies a restriction to a board

@group(0) @binding(0) var initial_board: texture_2d<u32>;
@group(0) @binding(1) var restriction:   texture_2d<u32>;

@group(0) @binding(2) var out_initial_board_restricted: texture_storage_2d<r32uint, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
    let initial_board_unrestricted: u32 = textureLoad(initial_board, vec2<i32>(global_thread_id.xy), 0).x;
    let restriction_mask:           u32 = textureLoad(restriction,   vec2<i32>(global_thread_id.xy), 0).x;

    let initial_board_restricted: u32 = initial_board_unrestricted & restriction_mask;
    textureStore(out_initial_board_restricted, vec2<i32>(global_thread_id.xy), vec4<u32>(initial_board_restricted));
}