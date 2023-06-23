use foundry::AsAny;

use super::colored_texture::ColoredTexture;

#[repr(C)]
#[allow(unused)]
#[derive(Debug, Clone, AsAny)]
pub struct PhongMaterialProperties {
    albedo: ColoredTexture, // default color
    metalic: ColoredTexture, // sininess color ?
}

impl PhongMaterialProperties {
    pub fn colored(mut self, color: glam::Vec3) -> Self {
        self.albedo.set_color(color);
        self
    }

    pub fn textured(mut self, texture_index: u32) -> Self {
        self.albedo.set_texture(texture_index);
        self
    }
}

impl Default for PhongMaterialProperties {
    fn default() -> Self {
        PhongMaterialProperties {
            albedo: ColoredTexture::color(glam::Vec3::ONE),
            metalic: ColoredTexture::color(glam::Vec3::ZERO),
        }
    }
}
