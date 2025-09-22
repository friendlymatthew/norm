#[derive(Debug)]
pub struct ShapeStack {
    shapes: Vec<Shape>,
}

impl ShapeStack {
    pub const fn new() -> Self {
        Self { shapes: vec![] }
    }

    pub fn push(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }

    // Find the topmost circle that contains the given point (in normalized coordinates)
    // Pushes the selected shape to the top of the stack
    pub fn find_shape_at_point(
        &mut self,
        x: f32,
        y: f32,
        window_width: u32,
        window_height: u32,
    ) -> Option<usize> {
        let top = self
            .shapes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, s)| match s {
                Shape::Circle {
                    x: cx,
                    y: cy,
                    radius,
                } => {
                    // Convert normalized coordinates to pixel coordinates for precise calculation
                    let pixel_x = x * window_width as f32;
                    let pixel_y = y * window_height as f32;
                    let circle_pixel_x = cx * window_width as f32;
                    let circle_pixel_y = cy * window_height as f32;
                    let circle_pixel_radius = radius * (window_width.min(window_height) as f32);

                    let distance = (pixel_x - circle_pixel_x).hypot(pixel_y - circle_pixel_y);

                    if distance <= circle_pixel_radius {
                        return Some(i);
                    }

                    None
                }
            });

        if let Some(i) = top {
            let selected = self.shapes.remove(i);
            self.shapes.push(selected);

            return Some(self.shapes.len() - 1);
        }

        None
    }

    // Move a circle to a new position (in normalized coordinates)
    pub fn move_circle(&mut self, index: usize, new_x: f32, new_y: f32) {
        if let Some(Shape::Circle { x, y, radius: _ }) = self.shapes.get_mut(index) {
            *x = new_x;
            *y = new_y;
        }
    }

    // Remove a circle by index
    pub fn remove_circle(&mut self, index: usize) {
        if index < self.shapes.len() {
            self.shapes.remove(index);
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Shape {
    Circle { x: f32, y: f32, radius: f32 },
}

pub fn compute_radius(from: (f32, f32), to: (f32, f32)) -> f32 {
    (from.0 - to.0).hypot(from.1 - to.1)
}
