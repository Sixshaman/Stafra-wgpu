struct FsInput
{
    @builtin(position) clip_position: vec4<f32>;
    @location(0)       texcoord:      vec2<f32>;
};

let FlagDrawOverlay: u32 = 0x01u;

struct ClickRuleFlags
{
    flags: u32;
}

@group(0) @binding(0) var          click_rule:       texture_2d<u32>;
@group(0) @binding(1) var<uniform> click_rule_flags: ClickRuleFlags;

@stage(fragment)
fn main(fin: FsInput) -> @location(0) vec4<f32>
{
    let click_rule_size        = vec2<f32>(textureDimensions(click_rule));
    let click_rule_coordinates = vec2<i32>(fin.texcoord * click_rule_size);

    let click_rule_val: u32 = textureLoad(click_rule, click_rule_coordinates, 0).x;
    let click_rule_color_rgb = f32(click_rule_val) * vec3<f32>(0.0, 1.0, 0.0);

    let click_rule_color = vec4<f32>(click_rule_color_rgb, 1.0);

    var overlay_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if((click_rule_flags.flags & FlagDrawOverlay) != 0u)
    {
        let truncated_size = click_rule_size - vec2<f32>(1.0, 1.0); //We don't use bottom and right texels

        let texcoord_corrected: vec2<f32> = fin.texcoord * click_rule_size / truncated_size;
        let cell_size = vec2<f32>(1.0, 1.0) / click_rule_size;

        let middle_vertical_line_distance:           f32 = abs(texcoord_corrected.x - 0.5);
        let middle_horizontal_line_distance:         f32 = abs(texcoord_corrected.y - 0.5);
        let top_left_bottom_right_diagonal_distance: f32 = abs(texcoord_corrected.x - texcoord_corrected.y);
        let top_right_bottom_left_diagonal_distance: f32 = abs(texcoord_corrected.x + texcoord_corrected.y - 1.0);

        let left_vertical_line_distance:     f32 = abs(texcoord_corrected.x -                     0.5  * cell_size.x);
        let right_vertical_line_distance:    f32 = abs(texcoord_corrected.x - (truncated_size.x - 0.5) * cell_size.x);
        let top_horizontal_line_distance:    f32 = abs(texcoord_corrected.x -                     0.5  * cell_size.x);
        let bottom_horizontal_line_distance: f32 = abs(texcoord_corrected.x - (truncated_size.x - 0.5) * cell_size.x);

        let edge_vertical_line_distance   = min(left_vertical_line_distance, right_vertical_line_distance);
        let edge_horizontal_line_distance = min(top_horizontal_line_distance, bottom_horizontal_line_distance);
        let vertical_line_distance        = min(middle_vertical_line_distance, edge_vertical_line_distance);
        let horizontal_line_distance      = min(middle_horizontal_line_distance, edge_horizontal_line_distance);
        let straight_line_distance        = min(vertical_line_distance, horizontal_line_distance);
        let diagonal_line_distance        = min(top_left_bottom_right_diagonal_distance, top_right_bottom_left_diagonal_distance);
        let line_distance                 = min(straight_line_distance, diagonal_line_distance);

        let line_width  = 0.005;
        let line_factor = smoothStep(line_width - line_distance, 0.0, line_width);

        overlay_color = overlay_color + vec4<f32>(line_factor, line_factor, line_factor, 0.0);
    }

	return click_rule_color;
}