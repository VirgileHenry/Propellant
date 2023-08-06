

/// Represents a texture with a color applied to it.
/// Most materials use this instead of colors, allowing for dynamic maps.
#[repr(C)]
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct ColoredTexture {
    color: glam::Vec3,
    texture_index: u32,
}

impl ColoredTexture {
    /// Create an color.
    pub fn color(color: glam::Vec3) -> Self {
        ColoredTexture {
            color,
            texture_index: 0, // 0 is reserved for white 1x1
        }
    }

    /// Create a texture.
    pub fn texture(texture_index: u32) -> Self {
        ColoredTexture {
            color: glam::Vec3::new(1., 1., 1.),
            texture_index,
        }
    }

    pub fn set_color(&mut self, color: glam::Vec3) {
        self.color = color;
    }

    pub fn set_texture(&mut self, texture_index: u32) {
        self.texture_index = texture_index;
    }
}