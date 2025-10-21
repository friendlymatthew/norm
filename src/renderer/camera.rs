#[derive(Debug, Clone, Copy)]
pub struct Camera {
    pub pan_x: f32,
    pub pan_y: f32,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            pan_x: 0.0,
            pan_y: 0.0,
            zoom: 1.0,
        }
    }

    pub fn pan(&mut self, delta_x: f32, delta_y: f32) {
        self.pan_x += delta_x / self.zoom;
        self.pan_y += delta_y / self.zoom;
    }

    // factor > 1.0 zooms in, factor < 1.0 zooms out
    pub fn zoom(&mut self, factor: f32) {
        self.zoom *= factor;
        // Clamp zoom to reasonable bounds
        self.zoom = self.zoom.clamp(0.1, 10.0);
    }

    pub fn _set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.1, 10.0);
    }

    pub fn reset(&mut self) {
        self.pan_x = 0.0;
        self.pan_y = 0.0;
        self.zoom = 1.0;
    }

    // generate a view-proj matrix for the camera
    pub fn view_projection_matrix(&self) -> [[f32; 4]; 4] {
        [
            [self.zoom, 0.0, 0.0, 0.0],
            [0.0, self.zoom, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [self.pan_x, self.pan_y, 0.0, 1.0],
        ]
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
