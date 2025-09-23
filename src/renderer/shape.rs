#[allow(dead_code)]
#[derive(Debug)]
pub enum Action {
    Move {
        shape_id: usize,
        orig_x: f32,
        orig_y: f32,
    },
    Draw {
        shape_id: usize,
    },
    Delete {
        shape_id: usize,
    },
}

#[derive(Debug, Default)]
pub struct RevisionStack {
    stack: Vec<Action>,

    pub shape_stack: ShapeStack,
}

impl RevisionStack {
    pub fn push_shape(&mut self, shape: Shape) {
        let id = self.shape_stack.push(shape);
        self.stack.push(Action::Draw { shape_id: id });
    }

    pub fn undo(&mut self) {
        if let Some(last_action) = self.stack.last() {
            match last_action {
                Action::Move {
                    // shape_id,
                    // orig_x,
                    // orig_y,
                    ..
                } => todo!(),
                Action::Draw { shape_id } => self.shape_stack.remove_shape_by_id(*shape_id),
                Action::Delete { shape_id: _ } => todo!(),
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ShapeStack {
    next_id: usize,
    shapes: Vec<(usize, Shape)>,
}

impl ShapeStack {
    pub fn new_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        id
    }

    pub fn push(&mut self, shape: Shape) -> usize {
        let id = self.new_id();

        self.shapes.push((id, shape));

        id
    }

    pub fn get_unchecked(&self, i: usize) -> &Shape {
        let res = unsafe { self.shapes.get_unchecked(i) };

        &res.1
    }

    pub const fn len(&self) -> usize {
        self.shapes.len()
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
            .find_map(|(i, (_shape_id, s))| match s {
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
        let (_, shape) = self
            .shapes
            .get_mut(index)
            .expect("index should always be valid");

        shape.update_position(new_x, new_y);
    }

    pub fn remove_shape(&mut self, index: usize) {
        self.shapes.remove(index);
    }

    pub fn remove_shape_by_id(&mut self, shape_id: usize) {
        if let Some(i) =
            self.shapes
                .iter()
                .enumerate()
                .find_map(|(j, (i, _))| if *i == shape_id { Some(j) } else { None })
        {
            self.remove_shape(i);
        }
    }

    pub fn shapes(&self) -> &[(usize, Shape)] {
        &self.shapes
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
            Shape::Circle { x, y, .. } => {
                *x = new_x;
                *y = new_y;
            }
        }
    }
}

pub fn compute_distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    (from.0 - to.0).hypot(from.1 - to.1)
}
