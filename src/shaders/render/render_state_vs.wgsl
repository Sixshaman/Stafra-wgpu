const fullscreen_triangle_positions = array<vec4<f32>, 3>
(
    vec4<f32>(-3.0, -1.0,  0.0,  1.0),
    vec4<f32>( 1.0, -1.0,  0.0,  1.0),
    vec4<f32>( 1.0,  3.0,  0.0,  1.0)
);

const fullscreen_triangle_texcoords = array<vec2<f32>, 3>
(
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0,  1.0),
    vec2<f32>( 1.0, -1.0)
);

struct VsOutput
{
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       texcoord:      vec2<f32>
};

@vertex
fn main(@builtin(vertex_index) vertex_id: u32) -> VsOutput
{
    var vout: VsOutput;

    if(vertex_id == 0u)
    {
        vout.clip_position = fullscreen_triangle_positions[0];
        vout.texcoord      = fullscreen_triangle_texcoords[0];
    }
    else if(vertex_id == 1u)
    {
        vout.clip_position = fullscreen_triangle_positions[1];
        vout.texcoord      = fullscreen_triangle_texcoords[1];
    }
    else if(vertex_id == 2u)
    {
        vout.clip_position = fullscreen_triangle_positions[2];
        vout.texcoord      = fullscreen_triangle_texcoords[2];
    }

    return vout;
}