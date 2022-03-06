@group(0) @binding(0) var board_tex:     texture_2d<f32>;
@group(0) @binding(1) var board_sampler: sampler;

struct FsInput
{
    @builtin(position) clip_position: vec4<f32>;
    @location(0)       texcoord:      vec2<f32>;
};

@stage(fragment)
fn main(fin: FsInput) -> @location(0) vec4<f32>
{
    //We store the stability texture in 2x2 quads with one RGBA channel for each element of the quad.
    //To display this correctly, we manually calculate the LOD to sample and mix the quad values
    let mip_0_size      = vec2<f32>(textureDimensions(board_tex, 0));
    let stability_color = vec4<f32>(1.0, 0.0, 1.0, 1.0);

	let texcoord_big   = fin.texcoord * mip_0_size;
	let d_texcoord_big = vec2<f32>(dpdx(texcoord_big.x), dpdx(texcoord_big.y)); //dfdx(y) and dfdy(x) are 0

	//Since wgpu doesn't support textureQueryLod, calculate the lod manually (as in https://www.khronos.org/registry/OpenGL/specs/gl/glspec46.core.pdf#section.8.14.1)
	let min_lod = 0.0;
	let max_lod = f32(textureNumLevels(board_tex) - 1);

    let base_lod = log2(max(d_texcoord_big.x, d_texcoord_big.y));
    let lod      = clamp(base_lod, min_lod, max_lod);

    let board_values:    vec4<f32> = textureSampleLevel(board_tex, board_sampler, fin.texcoord, lod);
    let lerp_parameters: vec2<f32> = fract(fin.texcoord * mip_0_size);
	if(lod < 0.00001 && any(d_texcoord_big > vec2<f32>(1.0)))
	{
		if(lerp_parameters.x < 0.5 && lerp_parameters.y < 0.5)
		{
			return stability_color * vec4<f32>(board_values.r, board_values.r, board_values.r, 1.0);
		}
		else if(lerp_parameters.x >= 0.5 && lerp_parameters.y < 0.5)
		{
			return stability_color * vec4<f32>(board_values.g, board_values.g, board_values.g, 1.0);
		}
		else if(lerp_parameters.x < 0.5 && lerp_parameters.y >= 0.5)
		{
			return stability_color * vec4<f32>(board_values.b, board_values.b, board_values.b, 1.0);
		}
		else if(lerp_parameters.x >= 0.5 && lerp_parameters.y >= 0.5)
		{
			return stability_color * vec4<f32>(board_values.a, board_values.a, board_values.a, 1.0);
		}
	}
	else
	{
		let vertical_average   = mix(board_values.rg,    board_values.ba,    lerp_parameters.y);
		let horizontal_average = mix(vertical_average.x, vertical_average.y, lerp_parameters.x);

		return stability_color * vec4<f32>(horizontal_average, horizontal_average, horizontal_average, 1.0);
	}

	return stability_color;
}