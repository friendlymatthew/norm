#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorToneUniform {
    grayscale: u32,
    sepia: u32,
    invert: u32,
}

impl ColorToneUniform {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn update(
        &mut self,
        new_grayscale_state: bool,
        new_sepia_state: bool,
        new_invert_state: bool,
    ) {
        self.grayscale = new_grayscale_state as u32;
        self.sepia = new_sepia_state as u32;
        self.invert = new_invert_state as u32;
    }
}
