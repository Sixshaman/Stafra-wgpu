#version 450

const vec4 near_plane_positions[3] = vec4[]
(
    vec4(-3.0f, -1.0f,  0.0f,  1.0f),
    vec4( 1.0f, -1.0f,  0.0f,  1.0f),
    vec4( 1.0f,  3.0f,  0.0f,  1.0f)
);

void main()
{
    gl_Position = near_plane_positions[gl_VertexIndex];
}