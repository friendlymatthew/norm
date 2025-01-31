// Vertex shader
struct ColorToneUniform {
    grayscale: u32,
    sepia: u32,
    invert: u32,
    gamma: u32,
};

struct BlurUniform {
    blur: u32,
    radius: u32,
    width: u32,
    height: u32,
    sharpen: u32,
    sharpen_factor: u32,
}

@group(1) @binding(0)
var<uniform> color_tone_uniform: ColorToneUniform;

@group(2) @binding(0)
var<uniform> blur_uniform: BlurUniform;

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

const PI: f32 = 22.0 / 7.0;

fn gaussian(offset: vec2<f32>) -> f32 {
    var GAUSSIAN_SIGMA: f32 = f32(blur_uniform.radius) * 0.25;
    return 1.0 / (2.0 * PI * GAUSSIAN_SIGMA * GAUSSIAN_SIGMA) * exp(-((offset.x * offset.x + offset.y * offset.y) / (2.0 * GAUSSIAN_SIGMA * GAUSSIAN_SIGMA)));
}

fn gaussian_blur(tex_coords: vec2<f32>, radius: f32, viewport_resolution: vec2<f32>) -> vec4<f32> {
    var acc = 0.0;
    var color = vec4<f32>();
    var weight = 0.0;

    for (var x = -1.0 * radius / 2.0; x < radius / 2.0; x = x + 1) {
        for (var y = -1.0 * radius / 2.0; y < radius / 2.0; y = y + 1) {
            var offset = vec2(x, y);
            weight = gaussian(offset);
            color = color + (textureSample(t_diffuse, s_diffuse, tex_coords + viewport_resolution * offset) * weight);
            acc = acc + weight;
        }
    }

    return color / acc;
}

fn box_blur(tex_coords: vec2<f32>, viewport_resolution: vec2<f32>) -> vec4<f32> {
    var acc_color = vec4<f32>(0.0);

    var ct = 0.0;

    for (var dx = -1.0; dx <= 1.0; dx = dx + 1.0) {
        for (var dy = -1.0; dy <= 1.0; dy = dy + 1.0) {
            acc_color += textureSample(t_diffuse, s_diffuse, tex_coords + viewport_resolution * vec2<f32>(dx, dy));
            ct = ct + 1.0;
        }
    }

    return acc_color / ct;
}

fn sharpen(tex_coords: vec2<f32>, sharpen_factor: f32, viewport_resolution: vec2<f32>) -> vec4<f32> {
    var center = textureSample(t_diffuse, s_diffuse, tex_coords);

    var up = textureSample(t_diffuse, s_diffuse, tex_coords + (vec2(0, 1) * viewport_resolution));
    var left = textureSample(t_diffuse, s_diffuse, tex_coords + (vec2(-1, 0) * viewport_resolution));
    var right = textureSample(t_diffuse, s_diffuse, tex_coords + (vec2(1, 0) * viewport_resolution));
    var down = textureSample(t_diffuse, s_diffuse, tex_coords + (vec2(0, -1) * viewport_resolution));

    return (1.0 + 4.0 * sharpen_factor) * center - sharpen_factor * (up + left + right + down);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var pixels = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    var viewport_resolution = 1.0 / vec2(f32(blur_uniform.width), f32(blur_uniform.height));

    if blur_uniform.sharpen == 1u {
        pixels = sharpen(in.tex_coords, f32(blur_uniform.sharpen_factor), viewport_resolution);
    }

    if blur_uniform.blur == 1u {
        pixels = gaussian_blur(in.tex_coords, f32(blur_uniform.radius), viewport_resolution);
    }

    if color_tone_uniform.gamma != 0u {
        // todo! modify the pixels to account for gamma
        // see https://www.w3.org/TR/2003/REC-PNG-20031110/#13Decoder-gamma-handling
    }

    if color_tone_uniform.grayscale == 1u {
        var y = (pixels.r * 0.29891 + pixels.g * 0.58661 + pixels.b * 0.11448);
        pixels = vec4(y, y, y, 1.0);
    }

    if color_tone_uniform.sepia == 1u {
        pixels = vec4<f32>(
            0.393 * pixels.r + 0.769 * pixels.g + 0.189 * pixels.b,
            0.349 * pixels.r + 0.686 * pixels.g + 0.168 * pixels.b,
            0.272 * pixels.r + 0.534 * pixels.g + 0.131 * pixels.b,
            0.30 * pixels.r + 0.59 * pixels.g + 0.11 * pixels.b
        );
    }

    if color_tone_uniform.invert == 1u {
        pixels = vec4<f32>(
            1.0 - pixels.r,
            1.0 - pixels.g,
            1.0 - pixels.b,
            pixels.a
        );
    }

    return pixels;
}
