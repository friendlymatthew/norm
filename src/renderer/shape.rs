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
        mouse_position_in_pixel_coords: Coordinate, // in pixel coordinates
        window_width: u32,
        window_height: u32,
    ) -> Option<usize> {
        let top = self
            .shapes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, (_shape_id, s))| match s {
                Shape::Circle(circle) => {
                    // convert normalized coordinates to pixel coordinates for precise calculation
                    let circle_in_pixel_coords =
                        circle.into_pixel_coordinate((window_width as f32, window_height as f32));

                    if compute_distance(
                        circle_in_pixel_coords.center(),
                        mouse_position_in_pixel_coords,
                    ) <= circle_in_pixel_coords.radius()
                    {
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

        shape.translate((new_x, new_y));
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
    Circle(Circle),
}

impl Shape {
    pub fn translate(&mut self, new_coord: Coordinate) {
        match self {
            Shape::Circle(circle) => {
                circle.translate(new_coord);
            }
        }
    }
}

pub fn compute_distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    (from.0 - to.0).hypot(from.1 - to.1)
}

// Stored as (x, y)
pub type Coordinate = (f32, f32);

#[derive(Debug)]
pub struct Circle {
    center: Coordinate,
    radius: f32,
}

impl Circle {
    pub fn from_pixel_coordinate(
        center: Coordinate,
        radius: f32,
        window_dimensions: (f32, f32),
    ) -> Self {
        let (w, h) = window_dimensions;
        let (cx, cy) = center;

        Self {
            center: (cx / w, cy / h),
            radius: radius / w.min(h),
        }
    }

    pub fn into_pixel_coordinate(&self, window_dimensions: (f32, f32)) -> Self {
        let (w, h) = window_dimensions;
        let (cx, cy) = self.center;

        Self {
            center: (cx * w, cy * h),
            radius: self.radius * w.min(h),
        }
    }

    pub fn center(&self) -> Coordinate {
        self.center
    }

    pub fn radius(&self) -> f32 {
        self.radius
    }

    pub fn translate(&mut self, new_center: Coordinate) {
        self.center = new_center;
    }
}
