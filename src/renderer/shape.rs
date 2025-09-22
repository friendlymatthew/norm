#[derive(Debug)]
pub struct RevisionStack {
    shapes: Vec<Shape>,
}

impl RevisionStack {
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

                    let distance =
                        compute_distance((pixel_x, pixel_y), (circle_pixel_x, circle_pixel_y));

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

    // Move a shape to a new position (in normalized coordinates)
    pub fn move_shape(&mut self, index: usize, new_x: f32, new_y: f32) {
        let shape = self
            .shapes
            .get_mut(index)
            .expect("index should always be valid");

        shape.update_position(new_x, new_y);
    }

    pub fn remove_shape(&mut self, index: usize) {
        self.shapes.remove(index);
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Shape {
    Circle { x: f32, y: f32, radius: f32 },
}

impl Shape {
    pub fn update_position(&mut self, new_x: f32, new_y: f32) {
        match self {
            Shape::Circle {
                x,
                y,
                radius: _radius,
            } => {
                *x = new_x;
                *y = new_y;
            }
        }
    }
}

pub fn compute_distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    (from.0 - to.0).hypot(from.1 - to.1)
}
