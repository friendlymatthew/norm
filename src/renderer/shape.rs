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

    pub fn _shapes(&self) -> &[Shape] {
        &self.shapes
    }

    // Find the topmost circle that contains the given point (in normalized coordinates)
    pub fn find_circle_at_point(&self, x: f32, y: f32) -> Option<usize> {
        // Search from the end (topmost/most recent) to find the top circle
        for (index, shape) in self.shapes.iter().enumerate().rev() {
            if let Shape::Circle { x: cx, y: cy, radius } = shape {
                let distance = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
                if distance <= *radius {
                    return Some(index);
                }
            }
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
