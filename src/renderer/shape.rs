use std::collections::BTreeMap;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ElementAction {
    Move {
        element_id: usize,
        original_pos: Coordinate,
    },
    Draw {
        element_id: usize,
    },
    Delete {
        element_id: usize,
    },
}

#[derive(Debug)]
pub struct Element {
    _id: usize,
    inner: Circle,
}

impl Element {
    pub const fn id(&self) -> usize {
        self._id
    }

    pub const fn inner(&self) -> &Circle {
        &self.inner
    }
}

#[derive(Debug, Default)]
pub struct EditorState {
    next_shape_id: usize,

    element_stack: Vec<Element>,
    revision_stack: Vec<ElementAction>,

    deleted_shapes: BTreeMap<usize, Circle>,
}

impl EditorState {
    pub const fn num_elements(&self) -> usize {
        self.element_stack.len()
    }

    pub fn elements(&self) -> &[Element] {
        &self.element_stack
    }

    pub const fn next_id(&mut self) -> usize {
        let id = self.next_shape_id;
        self.next_shape_id += 1;

        id
    }

    pub fn create_shape(&mut self, circle: Circle) -> usize {
        let _id = self.next_id();

        self.element_stack.push(Element { _id, inner: circle });
        self.revision_stack
            .push(ElementAction::Draw { element_id: _id });

        _id
    }

    // signals to the revision stack that the shape selected is being in the process of moving
    pub fn start_shape_translate(&mut self, element_id: usize) {
        if let Some(i) = self.get_element_index_by_id(element_id) {
            let element = unsafe { self.element_stack.get_unchecked(i) };
            let original_pos = element.inner.center();

            self.revision_stack.push(ElementAction::Move {
                element_id,
                original_pos,
            });
        }
    }

    pub fn translate_shape(&mut self, element_id: usize, new_coord: Coordinate) {
        if let Some(i) = self.get_element_index_by_id(element_id) {
            let element = unsafe { self.element_stack.get_unchecked_mut(i) };
            element.inner.translate(new_coord);
        }
    }

    pub fn get_element_index_by_id(&self, element_id: usize) -> Option<usize> {
        self.element_stack
            .iter()
            .enumerate()
            .find_map(|(i, &Element { _id, .. })| if _id == element_id { Some(i) } else { None })
    }

    pub fn get_element_by_point(
        &self,
        point: Coordinate,
        window_dimension: (f32, f32),
    ) -> Option<&Element> {
        self.element_stack.iter().rev().find(|e| {
            let circle = e.inner();
            let circle_in_pixel_coords = circle.convert_pixel_coordinate(window_dimension);

            compute_distance(circle_in_pixel_coords.center(), point)
                <= circle_in_pixel_coords.radius()
        })
    }

    pub fn remove_shape_by_id(&mut self, element_id: usize) {
        if let Some(i) = self.get_element_index_by_id(element_id) {
            self.remove_shape_by_index(i);
        }
    }

    fn remove_shape_by_index(&mut self, index: usize) {
        let Element { _id, inner } = self.element_stack.remove(index);
        self.revision_stack
            .push(ElementAction::Delete { element_id: _id });
        self.deleted_shapes.insert(_id, inner);
    }

    fn find_deleted_shape(&mut self, element_id: usize) -> Option<Circle> {
        self.deleted_shapes.remove(&element_id)
    }

    pub fn undo(&mut self) {
        if let Some(last_action) = self.revision_stack.last() {
            match last_action {
                ElementAction::Move {
                    element_id,
                    original_pos,
                } => self.translate_shape(*element_id, *original_pos),
                ElementAction::Draw { element_id } => self.remove_shape_by_id(*element_id),
                ElementAction::Delete { element_id } => {
                    if let Some(circle) = self.find_deleted_shape(*element_id) {
                        self.create_shape(circle);
                    }
                }
            }
        }
    }
}

pub fn compute_distance(from: Coordinate, to: Coordinate) -> f32 {
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

    pub fn convert_pixel_coordinate(&self, window_dimensions: (f32, f32)) -> Self {
        let (w, h) = window_dimensions;
        let (cx, cy) = self.center;

        Self {
            center: (cx * w, cy * h),
            radius: self.radius * w.min(h),
        }
    }

    pub const fn center(&self) -> Coordinate {
        self.center
    }

    pub const fn radius(&self) -> f32 {
        self.radius
    }

    pub const fn translate(&mut self, new_center: Coordinate) {
        self.center = new_center;
    }
}
