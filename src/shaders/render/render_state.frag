#version 450

#extension GL_EXT_samplerless_texture_functions: require

layout(set = 0, binding = 0) uniform texture2D boardTex;
layout(set = 0, binding = 1) uniform sampler   boardSampler;

layout(location = 0) in vec2 frag_texcoord;

layout(location = 0) out vec4 out_color;

void main()
{
	ivec2 mip0Size      = textureSize(boardTex, 0);
	vec4 stabilityColor = vec4(1.0f, 0.0f, 1.0f, 1.0f);

	vec2 texcoordBig  = frag_texcoord * vec2(mip0Size);
	vec2 dTexcoordBig = vec2(dFdx(texcoordBig.x), dFdy(texcoordBig.y));

	float lod = textureQueryLod(sampler2D(boardTex, boardSampler), frag_texcoord).y;

	vec4 boardValues = textureLod(sampler2D(boardTex, boardSampler), frag_texcoord, lod);
	vec2 lerpParams  = fract(frag_texcoord * vec2(mip0Size));
	
	if(lod < 0.00001f && any(greaterThan(dTexcoordBig, vec2(1.0f))))
	{
		if(lerpParams.x < 0.5f && lerpParams.y < 0.5f)
		{
			out_color = stabilityColor * vec4(vec3(boardValues.r), 1.0f);
		}
		else if(lerpParams.x >= 0.5f && lerpParams.y < 0.5f)
		{
			out_color = stabilityColor * vec4(vec3(boardValues.g), 1.0f);
		}
		else if(lerpParams.x < 0.5f && lerpParams.y >= 0.5f)
		{
			out_color = stabilityColor * vec4(vec3(boardValues.b), 1.0f);
		}
		else if(lerpParams.x >= 0.5f && lerpParams.y >= 0.5f)
		{
			out_color = stabilityColor * vec4(vec3(boardValues.a), 1.0f);
		}
	}
	else
	{
		vec2  vertAvg = mix(boardValues.rg, boardValues.ba, lerpParams.y);
		float horzAvg = mix(vertAvg.x, vertAvg.y, lerpParams.x);

		out_color = stabilityColor * vec4(vec3(horzAvg), 1.0f);
	}
}