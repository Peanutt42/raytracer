
struct RenderInfo {
	frame_counter: u32,
	// just a bool!
	normal_sky_color: u32,
}

@group(0) @binding(0) var output_texture: texture_2d<f32>;
@group(0) @binding(1) var output_sampler: sampler;
@group(1) @binding(0) var<uniform> render_info: RenderInfo;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// single fullscreen triangle
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), // lower left
        vec2<f32>( 3.0, -1.0), // far right (extends beyond clip-space)
        vec2<f32>(-1.0,  3.0)  // top (extends beyond clip-space)
    );

    var output: VertexOutput;
    let pos = positions[vertex_index];
    output.position = vec4<f32>(pos, 0.0, 1.0);
    // Map clip-space coordinates (-1 to 1) to UV coordinates (0 to 1).
    output.uv = pos * 0.5 + vec2<f32>(0.5, 0.5);
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let color = textureSample(output_texture, output_sampler, in.uv).xyz;
	let frame_count: f32 = f32(render_info.frame_counter);
    return vec4<f32>(color / frame_count, 1.0);
}