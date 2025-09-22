#[derive(Debug, Default)]
pub struct MouseState {
    pressed: bool,
    position_x: f32,
    position_y: f32,
    start_drag: Option<(f32, f32)>,
    selected_shape: Option<usize>, // index to the shape stack
    dragging_shape: bool,
    drag_offset: (f32, f32),
}

impl MouseState {
    pub(crate) const fn pressed(&self) -> bool {
        self.pressed
    }

    pub(crate) const fn set_pressed(&mut self, state: bool) {
        self.pressed = state;
    }

    pub(crate) const fn start_drag(&self) -> Option<(f32, f32)> {
        self.start_drag
    }

    pub(crate) const fn set_start_drag(&mut self, original_drag: Option<(f32, f32)>) {
        self.start_drag = original_drag;
    }

    pub(crate) const fn position(&self) -> (f32, f32) {
        (self.position_x, self.position_y)
    }

    pub(crate) const fn update_position(&mut self, x: f32, y: f32) {
        self.position_x = x;
        self.position_y = y;
    }

    pub(crate) const fn selected_shape(&self) -> Option<usize> {
        self.selected_shape
    }

    pub(crate) const fn set_selected_shape(&mut self, index: Option<usize>) {
        self.selected_shape = index;
    }

    pub(crate) const fn dragging_shape(&self) -> bool {
        self.dragging_shape
    }

    pub(crate) const fn set_dragging_shape(&mut self, dragging: bool) {
        self.dragging_shape = dragging;
    }

    pub(crate) const fn drag_offset(&self) -> (f32, f32) {
        self.drag_offset
    }

    pub(crate) const fn set_drag_offset(&mut self, offset: (f32, f32)) {
        self.drag_offset = offset;
    }
}
