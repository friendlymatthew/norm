// Vertex shader
struct ColorToneUniform {
    grayscale: u32,
    sepia: u32,
    invert: u32,
};

@group(1) @binding(0)
var<uniform> color_tone_uniform: ColorToneUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pixels = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    if color_tone_uniform.grayscale == 1u {
        var y = (pixels.r * 0.29891 + pixels.g * 0.58661 + pixels.b * 0.11448);
        return vec4<f32>(y, y, y, 1.0);
    }

    if color_tone_uniform.sepia == 1u {
        return vec4<f32>(
            0.393 * pixels.r + 0.769 * pixels.g + 0.189 * pixels.b,
            0.349 * pixels.r + 0.686 * pixels.g + 0.168 * pixels.b,
            0.272 * pixels.r + 0.534 * pixels.g + 0.131 * pixels.b,
            0.30 * pixels.r + 0.59 * pixels.g + 0.11 * pixels.b
        );
    }

    if color_tone_uniform.invert == 1u {
        return vec4<f32>(
            1.0 - pixels.r,
            1.0 - pixels.g,
            1.0 - pixels.b,
            pixels.a
        );
    }

    return pixels;
}
