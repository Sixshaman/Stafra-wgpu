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
	return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}