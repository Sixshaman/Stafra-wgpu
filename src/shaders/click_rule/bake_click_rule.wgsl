let click_rule_width  = 32u;
let click_rule_height = 32u;

struct ClickRuleData
{
    element_count:     atomic<u32>;
    radius:            atomic<u32>;
    padding:           vec2<u32>;
    enabled_positions: array<vec2<i32>, 1024>; //click_rule_width * click_rule_height
};

@group(0) @binding(0) var                      click_rule_tex:  texture_2d<u32>;
@group(0) @binding(1) var<storage, read_write> click_rule_data: ClickRuleData;

@stage(compute) @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_thread_id: vec3<u32>)
{
    if(any(global_thread_id.xy > vec2<u32>(click_rule_width - 1u, click_rule_height - 1u)))
    {
        return;
    }

    let click_rule_value = textureLoad(click_rule_tex, vec2<i32>(global_thread_id.xy), 0).x;
    if(click_rule_value != 0u)
    {
        let max_radius_x: i32 = (i32(click_rule_width)  - 1) / 2;
        let max_radius_y: i32 = (i32(click_rule_height) - 1) / 2;

        let click_rule_offset = vec2<i32>(global_thread_id.xy) - vec2<i32>(max_radius_x, max_radius_y);

        let next_index: u32 = atomicAdd(&click_rule_data.element_count, 1u);
        click_rule_data.enabled_positions[next_index] = click_rule_offset;

        let current_radius = u32(max(abs(click_rule_offset.x), abs(click_rule_offset.y))) + 1u;
        atomicMax(&click_rule_data.radius, current_radius);
    }
}
