// Vertex shader
struct SceneUniform {
    view_proj: mat4x4<f32>
};

@group(0) @binding(0)
var<uniform> scene: SceneUniform;

@group(1) @binding(0)
var<uniform> time: vec4<f32>;

struct VertexInput {
    @location(0) position  : vec3<f32>,
    @location(1) uv_coords : vec2<f32>,
}

struct InstanceInput {
    @location(2) size_offset : vec3<f32>,
    @location(3) velocity    : f32,
    @location(4) color       : vec3<f32>,
    @location(5) finisher    : u32,
    @location(6) hit         : f32,
};

struct VertexOutput {
    @builtin(position) clip_position : vec4<f32>,
    @location(0)       uv_coords     : vec2<f32>,
    @location(1)       color         : vec4<f32>,
    @location(2)       finisher      : u32,
}

@vertex
fn vs_main(
    vertex   : VertexInput,
    instance : InstanceInput,
) -> VertexOutput {
    let so = instance.size_offset;
    let model_matrix = mat4x4<f32>(
        vec4(so.x,  0.0, 0.0, 0.0),
        vec4( 0.0, so.y, 0.0, 0.0),
        vec4( 0.0,  0.0, 1.0, 0.0),
        vec4(so.z,  0.0, 0.0, 1.0),
    );

    let time_matrix = mat4x4<f32>(
        vec4(   1.0, 0.0, 0.0, 0.0),
        vec4(   0.0, 1.0, 0.0, 0.0),
        vec4(   0.0, 0.0, 1.0, 0.0),
        vec4(time.x, 0.0, 0.0, 1.0),
    );

    var out: VertexOutput;
    out.clip_position = model_matrix * vec4<f32>(vertex.position, 1.0);
    out.clip_position = time_matrix * out.clip_position;

    if instance.hit != 0.0 {
        // out.clip_position.y -= 10000.0;
        var offset = 0.0;

        let p = (time_matrix * model_matrix) * vec4<f32>(-0.5, 0.0, 0.0, 1.0);


        let hit_matrix = mat4x4<f32>(
            vec4(         1.0, 0.0, 0.0, 0.0),
            vec4(         0.0, 1.0, 0.0, 0.0),
            vec4(         0.0, 0.0, 1.0, 0.0),
            vec4(instance.hit, 0.0, 0.0, 1.0),
        );
        let h = (hit_matrix * model_matrix) * vec4<f32>(-0.5, 0.0, 0.0, 1.0);


        if (p * instance.velocity).x - (h * instance.velocity).x < 0.0 {
            let height = 12.5;
            let intensity = 0.36;

            offset = pow((p.x - h.x) * intensity + height, 2.0) * -1.0 + (height * height);
        } else {
            offset = 0.0;
        }

        out.clip_position.y -= offset;
    }

    out.clip_position.x *= instance.velocity;

    out.clip_position = scene.view_proj * out.clip_position;
    out.uv_coords = vertex.uv_coords;
    out.color = vec4(instance.color, 1.0);
    out.finisher = instance.finisher;

    return out;
}

// Fragment shader
@group(2) @binding(0) var t0 : texture_2d<f32>;
@group(2) @binding(1) var s0 : sampler;

@group(3) @binding(0) var t1 : texture_2d<f32>;
@group(3) @binding(1) var s1 : sampler;

@group(4) @binding(0) var t2 : texture_2d<f32>;
@group(4) @binding(1) var s2 : sampler;

@group(5) @binding(0) var t3 : texture_2d<f32>;
@group(5) @binding(1) var s3 : sampler;

fn to_srgb(srgba: vec4<f32>) -> vec4<f32> {
    let srgb = srgba.rgb;
    let cutoff = srgb < vec3<f32>(0.04045);
    let lower = srgb / vec3<f32>(12.92);
    let higher = pow((srgb + vec3<f32>(0.055)) / vec3<f32>(1.055), vec3<f32>(2.4));
    return vec4(select(higher, lower, cutoff), srgba.a);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_finisher = textureSample(t2, s2, in.uv_coords);
    let overlay_finisher = textureSample(t3, s3, in.uv_coords);
    let texture_circle = textureSample(t0, s0, in.uv_coords);
    let overlay_circle = textureSample(t1, s1, in.uv_coords);
    if in.finisher == u32(0) {
        let out = overlay_circle              * overlay_circle.a
                + (texture_circle * in.color) * (1.0 - overlay_circle.a);

        return to_srgb(out);
    } else {
        let out = overlay_finisher              * overlay_finisher.a
                + (texture_finisher * in.color) * (1.0 - overlay_finisher.a);

        return to_srgb(out);
    }
}
