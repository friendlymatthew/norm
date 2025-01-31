#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FeatureUniform {
    grayscale: u32,
    // sepia: u32,
    invert: u32,
    gamma: u32,
    blur: u32,
    blur_radius: u32,
    width: u32,
    height: u32,
    sharpen: u32,
    sharpen_factor: u32,
    edge_detect: u32,
}

impl FeatureUniform {
    pub(crate) const fn new(width: u32, height: u32, gamma: u32) -> Self {
        Self {
            grayscale: 0,
            // sepia: 0,
            invert: 0,
            gamma,
            width,
            height,
            blur: 0,
            blur_radius: 21,
            sharpen: 0,
            sharpen_factor: 16,
            edge_detect: 0,
        }
    }

    pub(crate) fn reset_features(&mut self) {
        self.grayscale = 0;
        // self.sepia = 0;
        self.invert = 0;
        self.blur = 0;
        self.sharpen = 0;
        self.edge_detect = 0;
    }
}

impl FeatureUniform {
    pub(crate) const fn grayscale(&self) -> bool {
        self.grayscale == 1
    }

    // pub(crate) const fn sepia(&self) -> bool {
    //     self.sepia == 1
    // }

    pub(crate) const fn invert(&self) -> bool {
        self.invert == 1
    }

    pub(crate) fn toggle_grayscale(&mut self) {
        self.grayscale = !self.grayscale() as u32;
    }

    // pub(crate) fn toggle_sepia(&mut self) {
    //     self.sepia = !self.sepia() as u32;
    // }

    pub(crate) fn toggle_invert(&mut self) {
        self.invert = !self.invert() as u32;
    }
}

impl FeatureUniform {
    const MAX_BLUR_RADIUS: u32 = 39;
    const MIN_BLUR_RADIUS: u32 = 3;

    pub(crate) const fn blur(&self) -> bool {
        self.blur == 1
    }

    pub(crate) fn toggle_blur(&mut self) {
        self.blur = !self.blur() as u32;
    }

    pub(crate) fn increase_blur_radius(&mut self) {
        self.blur_radius = (self.blur_radius + 2).min(Self::MAX_BLUR_RADIUS);
    }

    pub(crate) fn decrease_blur_radius(&mut self) {
        self.blur_radius = (self.blur_radius - 2).max(Self::MIN_BLUR_RADIUS);
    }
}

impl FeatureUniform {
    const MAX_SHARPEN_FACTOR: u32 = 40;
    const MIN_SHARPEN_FACTOR: u32 = 1;

    pub(crate) const fn sharpen(&self) -> bool {
        self.sharpen == 1
    }

    pub(crate) fn toggle_sharpen(&mut self) {
        self.sharpen = !self.sharpen() as u32;
    }

    pub(crate) fn increase_sharpen_factor(&mut self) {
        self.sharpen_factor = (self.sharpen_factor + 1).min(Self::MAX_SHARPEN_FACTOR);
    }

    pub(crate) fn decrease_sharpen_factor(&mut self) {
        self.sharpen_factor = (self.sharpen_factor - 1).max(Self::MIN_SHARPEN_FACTOR);
    }
}

impl FeatureUniform {
    pub(crate) const fn edge_detect(&self) -> bool {
        self.edge_detect == 1
    }

    pub(crate) fn toggle_edge_detect(&mut self) {
        self.edge_detect = !self.edge_detect() as u32;
    }
}
