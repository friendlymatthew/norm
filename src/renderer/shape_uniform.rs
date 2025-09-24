use crate::renderer::shape::Circle;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeUniform {
    width: u32,
    height: u32,
    num_circles: u32,
    selected_circle: u32, // Index of selected circle (u32::MAX = none)
}

impl ShapeUniform {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            num_circles: 0,
            selected_circle: u32::MAX,
        }
    }

    pub const fn update_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub const fn set_num_circles(&mut self, count: u32) {
        self.num_circles = count;
    }

    pub fn set_selected_circle(&mut self, selected: Option<usize>) {
        self.selected_circle = selected.map(|i| i as u32).unwrap_or(u32::MAX);
    }
}

// Circle data for storage buffer
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleData {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub _padding: f32,
}

impl From<&Circle> for CircleData {
    fn from(circle: &Circle) -> Self {
        let (x, y) = circle.center();
        let radius = circle.radius();

        Self {
            x,
            y,
            radius,
            _padding: 0.0,
        }
    }
}

pub const MAX_CIRCLES: usize = 256;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleBuffer {
    pub circles: [CircleData; MAX_CIRCLES],
}
