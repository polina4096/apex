// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct InstanceInput {
    @location(3) model_matrix_0 : vec4<f32>,
    @location(4) model_matrix_1 : vec4<f32>,
    @location(5) model_matrix_2 : vec4<f32>,
    @location(6) model_matrix_3 : vec4<f32>,
    @location(7) color          : vec4<f32>,
};

struct VertexInput {
    @location(0) position  : vec3<f32>,
    @location(1) uv_coords : vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0)       uv_coords     : vec2<f32>,
    @location(1)       color         : vec4<f32>,
}

@vertex
fn vs_main(
    vertex   : VertexInput,
    instance : InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.uv_coords = vertex.uv_coords;
    out.color = instance.color;
    return out;
}

// Fragment shader
@group(1) @binding(0) var t0: texture_2d<f32>;
@group(1) @binding(1) var s0: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t0, s0, in.uv_coords) * in.color;
}
