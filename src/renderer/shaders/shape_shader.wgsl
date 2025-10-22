// Shape rendering shader - draws circles and other shapes to a render texture

struct ShapeUniform {
    width: u32,
    height: u32,
    num_circles: u32,
    selected_circle: u32, // Index of selected circle (0xFFFFFFFF = none)
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

        // Convert normalized coordinates back to pixel coordinates
        let circle_center_pixels = vec2<f32>(
            circle.x * f32(shape_uniform.width),
            circle.y * f32(shape_uniform.height)
        );
        let circle_radius_pixels = circle.radius * f32(min(shape_uniform.width, shape_uniform.height));

        let distance_to_center = distance(pixel_coords, circle_center_pixels);

        if distance_to_center <= circle_radius_pixels {
            // Filled circle with anti-aliasing
            let edge_softness = 1.0;
            let alpha = 1.0 - smoothstep(circle_radius_pixels - edge_softness, circle_radius_pixels, distance_to_center);

            // Cyan filled circle
            let circle_color = vec4<f32>(0.0, 1.0, 1.0, alpha);

            // Alpha blend this circle over existing content
            color = mix(color, circle_color, alpha);
        }

        // If this circle is selected, draw a yellow outline
        if i == shape_uniform.selected_circle {
            let outline_thickness = 3.0;
            let outline_distance = abs(distance_to_center - circle_radius_pixels);

            if outline_distance <= outline_thickness {
                let outline_alpha = 1.0 - smoothstep(0.0, outline_thickness, outline_distance);
                let yellow_outline = vec4<f32>(1.0, 1.0, 0.0, outline_alpha);

                // Blend the yellow outline over existing color
                color = mix(color, yellow_outline, outline_alpha);
            }
        }
    }

    return color;
}