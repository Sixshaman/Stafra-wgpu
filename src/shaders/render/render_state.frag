#version 450

layout(set = 0, binding = 0) uniform texture2D boardTex;
layout(set = 0, binding = 1) uniform sampler   boardSampler;

layout(location = 0) in vec2 frag_texcoord;

layout(location = 0) out vec4 out_color;

void main()
{
	float boardVal       = texture(sampler2D(boardTex, boardSampler), frag_texcoord).x;
	vec4  stabilityColor = vec4(1.0f, 0.0f, 1.0f, 1.0f);

	out_color = stabilityColor * boardVal;
}