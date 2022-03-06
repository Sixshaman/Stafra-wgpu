//Clears the stability texture

@group(0) @binding(0) var out_initial_stability: texture_storage_2d<r32uint, write>;

@stage(compute) @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
    //Pack a uint32-encoded quad    from a bvec4-encoded quad:
    // (Bits 0-7)   (Bits 8-15)     (values.x) (values.y)
    // (Bits 16-23) (Bits 24-31)    (values.z) (values.w)
    let packed_quad: u32 = (1u <<  0u) | (1u <<  8u)
                         | (1u << 16u) | (1u << 24u);

    //Masks for halves of 2x2 quad, encoded as
    // (Bits 0-7)   (Bits 8-15)
    // (Bits 16-23) (Bits 24-31)
    let right_quad_mask:  u32 = 0xff00ff00u;
    let bottom_quad_mask: u32 = 0xffff0000u;

	let right_bottom_coordinates = vec2<u32>(textureDimensions(out_initial_stability) + vec2<i32>(-1, -1)); //Board size is always 2^n - 1. Mask out the bottom and right edge
	let on_right_bottom: vec2<bool> = (global_thread_id.xy == right_bottom_coordinates);

	let right_bottom_mask = vec2<u32>(right_quad_mask, bottom_quad_mask) * vec2<u32>(on_right_bottom);
    textureStore(out_initial_stability, vec2<i32>(global_thread_id.xy), vec4<u32>(packed_quad & ~(right_bottom_mask.x | right_bottom_mask.y)));
}