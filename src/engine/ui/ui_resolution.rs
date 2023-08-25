
#[derive(Debug, Clone, Copy)]
pub struct UiResolution {
    resolution: f32,
    screen_size: glam::Vec2,
}

impl Default for UiResolution {
    fn default() -> Self {
        UiResolution { 
            resolution: 1.0,
            screen_size: glam::vec2(1080., 19020.,),
        }
    }
}

impl UiResolution {
    pub fn new(resolution: f32, screen_size: glam::Vec2) -> UiResolution {
        UiResolution { resolution, screen_size, }
    }
    
    pub fn scale_factor(&self) -> glam::Vec2 {
        self.resolution / self.screen_size
    }

    pub fn screen_size(&self) -> glam::Vec2 {
        self.screen_size
    }

    pub fn resolution(&self) -> f32 {
        self.resolution
    }

    pub fn set_window_size(&mut self, new_size: glam::Vec2) {
        self.screen_size = new_size;
    }
}