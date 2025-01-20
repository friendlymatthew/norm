#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurUniform {
    blur: u32,
    pub(crate) radius: u32,
}

impl BlurUniform {
    pub(crate) const fn new() -> Self {
        Self { blur: 0, radius: 7 }
    }

    pub(crate) fn update(&mut self, new_blur_state: bool, new_radius: u32) {
        self.blur = new_blur_state as u32;
        self.radius = new_radius;
    }
}
