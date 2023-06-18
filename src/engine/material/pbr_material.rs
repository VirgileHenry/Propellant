use foundry::AsAny;

use crate::small_id;

#[repr(C)]
#[allow(unused)]
#[derive(Debug, Clone, AsAny)]
pub struct PbrMaterialProperties {
    color: glam::Vec3,
    smoothness: f32,
    metallic: f32,
    texture_id: u32,
    texture_index: u32,
    padding: u32,
}

impl PbrMaterialProperties {
    pub fn colored(self, color: glam::Vec3) -> Self {
        PbrMaterialProperties {
            color,
            ..self
        }
    }
}

impl Default for PbrMaterialProperties {
    fn default() -> Self {
        PbrMaterialProperties {
            color: glam::Vec3::new(1.0, 1.0, 1.0),
            smoothness: 0.0,
            metallic: 0.0,
            texture_id: small_id("default"),
            texture_index: 0,
            padding: 0,
        }
    }
}
