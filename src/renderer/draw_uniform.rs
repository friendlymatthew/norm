#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawUniform {
    pub crosshair: u32,
    pub circle_center_x: f32,
    pub circle_center_y: f32,
    pub circle_radius: f32,
    pub camera_view_proj: [[f32; 4]; 4],
}

impl Default for DrawUniform {
    fn default() -> Self {
        Self::new()
    }
}

impl DrawUniform {
    pub const fn new() -> Self {
        Self {
            crosshair: 0,
            circle_center_x: 0.0,
            circle_center_y: 0.0,
            circle_radius: 0.0,
            camera_view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub(crate) const fn _update_camera(&mut self, view_proj: [[f32; 4]; 4]) {
        self.camera_view_proj = view_proj;
    }

    pub(crate) const fn crosshair(&self) -> bool {
        self.crosshair == 1
    }

    pub(crate) const fn toggle_crosshair(&mut self) {
        self.crosshair = !self.crosshair() as u32;
    }

    pub(crate) const fn set_circle_center(&mut self, x: f32, y: f32) {
        self.circle_center_x = x;
        self.circle_center_y = y;
    }

    pub(crate) const fn set_circle_radius(&mut self, radius: f32) {
        self.circle_radius = radius;
    }
}
