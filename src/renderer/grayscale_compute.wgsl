// Compute shader for grayscale conversion

@group(0)
@binding(0)
var input_texture: texture_2d<f32>;

@group(0)
@binding(1)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);
    let dimensions = textureDimensions(input_texture);

    // Bounds check: exit early if outside image
    if (coords.x >= i32(dimensions.x) || coords.y >= i32(dimensions.y)) {
        return;
    }

    let color = textureLoad(input_texture, coords, 0);

    // Convert to grayscale using luminance formula
    let y = color.r * 0.29891 + color.g * 0.58661 + color.b * 0.11448;
    let gray = vec4<f32>(y, y, y, color.a);

    textureStore(output_texture, coords, gray);
}
