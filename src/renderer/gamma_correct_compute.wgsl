// a compute shader for gamma correction

@group(0) 
@binding(0)
var input_texture: texture_2d<f32>;

@group(0) 
@binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(0)
@binding(2)
var<uniform> gamma: u32;

@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);
    let dimensions = textureDimensions(input_texture);

    if (coords.x >= i32(dimensions.x) || coords.y >= i32(dimensions.y)) {
        return;
    }

    var color = textureLoad(input_texture, coords, 0);

    // If gamma is 0, just pass through (no correction)
    if (gamma != 0u) {
        let gamma_value = f32(gamma) / 100000.0;
        let inv_gamma = 1.0 / gamma_value;

        color = vec4<f32>(
            pow(color.r, inv_gamma),
            pow(color.g, inv_gamma),
            pow(color.b, inv_gamma),
            color.a
        );
    }

    textureStore(output_texture, coords, color);

}

