struct QuadUniform {
    over_edge: u32,
};


@group(0) @binding(0)
var<uniform> quad_uniform: QuadUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if quad_uniform.over_edge == 1u {
        return vec4<f32>(0.5);
    }

    return vec4<f32>(in.color, 0.5);
}