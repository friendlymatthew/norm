#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorToneUniform {
    grayscale: u32,
    sepia: u32,
}

impl ColorToneUniform {
    pub(crate) const fn new() -> Self {
        Self {
            grayscale: 0,
            sepia: 0,
        }
    }

    pub(crate) fn update(&mut self, new_grayscale_state: bool, new_sepia_state: bool) {
        self.grayscale = new_grayscale_state as u32;
        self.sepia = new_sepia_state as u32;
    }
}
