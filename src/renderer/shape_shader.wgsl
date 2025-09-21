// Shape rendering shader - draws circles and other shapes to a render texture

struct ShapeUniform {
    width: u32,
    height: u32,
    num_circles: u32,
    _padding: u32,
}

struct CircleData {
    x: f32,
    y: f32,
    radius: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> shape_uniform: ShapeUniform;

@group(0) @binding(1)
var<storage, read> circles: array<CircleData>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(model.position.x, model.position.y, model.position.z, 1.0);
    out.tex_coords = model.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Start with transparent background
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    let pixel_coords = in.tex_coords * vec2<f32>(f32(shape_uniform.width), f32(shape_uniform.height));

    // Render all circles
    for (var i = 0u; i < shape_uniform.num_circles; i = i + 1u) {
        let circle = circles[i];
        let distance_to_center = distance(pixel_coords, vec2<f32>(circle.x, circle.y));

        if distance_to_center <= circle.radius {
            // Circle outline - make it 2 pixels thick
            if distance_to_center >= circle.radius - 2.0 {
                color = vec4<f32>(0.0, 1.0, 1.0, 1.0); // Cyan outline
            }
        }
    }

    return color;
}