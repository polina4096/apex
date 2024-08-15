struct VertexInput {
    @location(0) position  : vec3<f32>,
    @location(1) uv_coords : vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0)       uv_coords     : vec2<f32>,
}

@group(1) @binding(0)
var<uniform> fade_time: f32;

@group(2) @binding(0)
var<uniform> border_thickness: vec2<f32>;

@vertex
fn vs_main(
    vertex: VertexInput,
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(vertex.position, 1.0);
    out.uv_coords = vertex.uv_coords;
    return out;
}

// Fragment shader
@group(0) @binding(0) var t0: texture_2d<f32>;
@group(0) @binding(1) var s0: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if in.uv_coords.x < border_thickness.x
    || in.uv_coords.y < border_thickness.y
    || in.uv_coords.x > 1.0 - border_thickness.x
    || in.uv_coords.y > 1.0 - border_thickness.y
    {
        return vec4<f32>(1.0, 1.0, 1.0, 1.0 - fade_time * 2.0);
    }

    return textureSample(t0, s0, in.uv_coords);
}
