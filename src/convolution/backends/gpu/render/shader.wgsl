//! Fullscreen vertex shader like Bevy does, see: https://github.com/bevyengine/bevy/blob/main/crates/bevy_core_pipeline/src/fullscreen_vertex_shader/fullscreen.wgsl 

struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
};

@vertex
fn fullscreen_vertex_shader(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOutput {
    let uv = vec2<f32>(f32(vertex_index >> 1u), f32(vertex_index & 1u)) * 2.0;
    let clip_position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);

    return FullscreenVertexOutput(clip_position, uv);
}

@group(0) @binding(0)
var<uniform> pixel_size: vec2<f32>;
@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(2)
var s_diffuse: sampler;



@fragment
fn frag_shader(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
	var uv = in.uv;

	if (in.uv.x > 0.5) {
		uv.y = 1. - uv.y;
	}

	return textureSample(t_diffuse, s_diffuse, uv);
}