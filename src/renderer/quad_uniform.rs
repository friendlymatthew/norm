#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadUniform {
    over_quad: u32,
}

impl QuadUniform {
    pub(crate) const fn new() -> Self {
        Self { over_quad: 0 }
    }
}

impl QuadUniform {
    pub(crate) const fn over_quad(&self) -> bool {
        self.over_quad == 1
    }

    pub(crate) fn set_over_quad(&mut self, state: bool) {
        self.over_quad = state as u32;
    }
}
