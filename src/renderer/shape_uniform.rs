use crate::renderer::shape::Shape;

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShapeUniform {
    width: u32,
    height: u32,
    num_circles: u32,
    _padding: u32,
}

impl ShapeUniform {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            num_circles: 0,
            _padding: 0,
        }
    }

    pub fn update_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    pub fn set_num_circles(&mut self, count: u32) {
        self.num_circles = count;
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

impl From<&Shape> for CircleData {
    fn from(shape: &Shape) -> Self {
        match shape {
            Shape::Circle { x, y, radius } => Self {
                x: *x,
                y: *y,
                radius: *radius,
                _padding: 0.0,
            },
        }
    }
}

pub const MAX_CIRCLES: usize = 256;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleBuffer {
    pub circles: [CircleData; MAX_CIRCLES],
}