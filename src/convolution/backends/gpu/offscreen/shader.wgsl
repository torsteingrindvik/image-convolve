//! Fullscreen vertex shader like Bevy does, see: https://github.com/bevyengine/bevy/blob/main/crates/bevy_core_pipeline/src/fullscreen_vertex_shader/fullscreen.wgsl 

struct FullscreenVertexOutput {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    uv: vec2<f32>,
};

@vertex
fn vs_fullscreen(@builtin(vertex_index) vertex_index: u32) -> FullscreenVertexOutput {
    let uv = vec2<f32>(f32(vertex_index >> 1u), f32(vertex_index & 1u)) * 2.0;
    let clip_position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);

    return FullscreenVertexOutput(clip_position, uv);
}

@group(0) @binding(0)
var t: texture_2d<f32>;
@group(0)@binding(1)
var s: sampler;


@fragment
fn fs_identity(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
	var rgb = vec3<f32>(0.);

	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(-1, -1)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(0, -1)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(1, -1)).rgb;

	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(-1, 0)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(0, 0)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(1, 0)).rgb;

	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(-1, 1)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(0, 1)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(1, 1)).rgb;

	// normalization
	// rgb /= 1.;

	return vec4(rgb, 1.);
}

@fragment
fn fs_edge_detection1(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
	var rgb = vec3<f32>(0.);

	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(-1, -1)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(0, -1)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(1, -1)).rgb;

	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(-1, 0)).rgb;
	rgb += 4. * textureSample(t, s, in.uv, vec2<i32>(0, 0)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(1, 0)).rgb;

	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(-1, 1)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(0, 1)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(1, 1)).rgb;

	// normalization
	// rgb /= 1.;

	return vec4(rgb, 1.);
}

@fragment
fn fs_edge_detection2(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
	var rgb = vec3<f32>(0.);

	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(-1, -1)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(0, -1)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(1, -1)).rgb;

	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(-1, 0)).rgb;
	rgb += 8. * textureSample(t, s, in.uv, vec2<i32>(0, 0)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(1, 0)).rgb;

	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(-1, 1)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(0, 1)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(1, 1)).rgb;

	// normalization
	// rgb /= 1.;

	return vec4(rgb, 1.);
}

@fragment
fn fs_sharpen(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
	var rgb = vec3<f32>(0.);

	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(-1, -1)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(0, -1)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(1, -1)).rgb;

	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(-1, 0)).rgb;
	rgb += 5. * textureSample(t, s, in.uv, vec2<i32>(0, 0)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(1, 0)).rgb;

	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(-1, 1)).rgb;
	rgb += -1. * textureSample(t, s, in.uv, vec2<i32>(0, 1)).rgb;
	// rgb += 0. * textureSample(t, s, in.uv, vec2<i32>(1, 1)).rgb;

	// normalization
	// rgb /= 1.;

	return vec4(rgb, 1.);
}

@fragment
fn fs_box_blur(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
	var rgb = vec3<f32>(0.);

	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(-1, -1)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(0, -1)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(1, -1)).rgb;

	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(-1, 0)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(0, 0)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(1, 0)).rgb;

	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(-1, 1)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(0, 1)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(1, 1)).rgb;

	// normalization
	rgb /= 9.;

	return vec4(rgb, 1.);
}

@fragment
fn fs_gaussian_blur(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
	var rgb = vec3<f32>(0.);

	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(-1, -1)).rgb;
	rgb += 2. * textureSample(t, s, in.uv, vec2<i32>(0, -1)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(1, -1)).rgb;

	rgb += 2. * textureSample(t, s, in.uv, vec2<i32>(-1, 0)).rgb;
	rgb += 4. * textureSample(t, s, in.uv, vec2<i32>(0, 0)).rgb;
	rgb += 2. * textureSample(t, s, in.uv, vec2<i32>(1, 0)).rgb;

	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(-1, 1)).rgb;
	rgb += 2. * textureSample(t, s, in.uv, vec2<i32>(0, 1)).rgb;
	rgb += 1. * textureSample(t, s, in.uv, vec2<i32>(1, 1)).rgb;

	// normalization
	rgb /= 16.;

	return vec4(rgb, 1.);
}