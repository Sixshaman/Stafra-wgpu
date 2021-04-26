#version 450

const vec4 fullscreen_triangle_positions[3] = vec4[]
(
    vec4(-3.0f, -1.0f,  0.0f,  1.0f),
    vec4( 1.0f, -1.0f,  0.0f,  1.0f),
    vec4( 1.0f,  3.0f,  0.0f,  1.0f)
);

const vec2 fullscreen_triangle_texcoords[3] = vec2[]
(
    vec2(-1.0f,  0.0f),
    vec2( 1.0f,  0.0f),
    vec2( 1.0f,  2.0f)
);

layout(location = 0) out vec2 frag_texcoord;

void main()
{
    gl_Position   = fullscreen_triangle_positions[gl_VertexIndex];
    frag_texcoord = fullscreen_triangle_texcoords[gl_VertexIndex]; 
}