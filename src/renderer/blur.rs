#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BlurUniform {
    blur: u32,
    pub(crate) radius: u32,
    width: u32,
    height: u32,
    sharpen: u32,
    pub(crate) sharpen_factor: u32,
}

impl BlurUniform {
    pub(crate) const fn new(width: u32, height: u32) -> Self {
        Self {
            blur: 0,
            radius: 21,
            width,
            height,
            sharpen: 0,
            sharpen_factor: 16,
        }
    }

    pub(crate) fn update(
        &mut self,
        new_blur_state: bool,
        new_radius: u32,
        new_width: u32,
        new_height: u32,
        new_sharpen_state: bool,
        new_sharpen_factor: u32,
    ) {
        self.blur = new_blur_state as u32;
        self.radius = new_radius;
        self.width = new_width;
        self.height = new_height;
        self.sharpen = new_sharpen_state as u32;
        self.sharpen_factor = new_sharpen_factor
    }
}
