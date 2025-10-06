// Compute shader for simple color corrections: gamma, grayscale, invert
// These are cheap per-pixel operations that don't need neighboring pixels

struct FeatureUniform {
    grayscale: u32,
    invert: u32,
    gamma: u32,
    blur: u32,
    blur_radius: u32,
    width: u32,
    height: u32,
    sharpen: u32,
    sharpen_factor: u32,
    edge_detect: u32,
    _padding_1: u32,
    _padding_2: u32,
    transform: mat4x4<f32>,
}

@group(0) @binding(0)
var input_texture: texture_2d<f32>;

@group(0) @binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(2)
var<uniform> feature_uniform: FeatureUniform;

@compute @workgroup_size(16, 16)
fn cs_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);

    if (coords.x >= i32(feature_uniform.width) || coords.y >= i32(feature_uniform.height)) {
        return;
    }

    var color = textureLoad(input_texture, coords, 0);

    if (feature_uniform.gamma != 0u) {
        let gamma_value = f32(feature_uniform.gamma) / 100000.0;
        let inv_gamma = 1.0 / gamma_value;

        color = vec4<f32>(
            pow(color.r, inv_gamma),
            pow(color.g, inv_gamma),
            pow(color.b, inv_gamma),
            color.a
        );
    }

    if (feature_uniform.grayscale == 1u) {
        let y = color.r * 0.29891 + color.g * 0.58661 + color.b * 0.11448;
        color = vec4<f32>(y, y, y, color.a);
    }

    if (feature_uniform.invert == 1u) {
        color = vec4<f32>(
            1.0 - color.r,
            1.0 - color.g,
            1.0 - color.b,
            color.a
        );
    }

    textureStore(output_texture, coords, color);
}
