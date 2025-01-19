#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GrayscaleUniform {
    grayscale: u32,
}

impl GrayscaleUniform {
    pub(crate) const fn new() -> Self {
        Self { grayscale: 0 }
    }

    pub(crate) fn toggle_grayscale(&mut self, new_state: bool) {
        self.grayscale = new_state as u32;
    }
}
