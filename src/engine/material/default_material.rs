
#[repr(C)]
#[allow(unused)]
pub struct DefaultMaterial {
    color: glam::Vec3,
    smoothness: f32,
    metallic: f32,
}

impl Default for DefaultMaterial {
    fn default() -> Self {
        DefaultMaterial {
            color: glam::Vec3::new(1.0, 1.0, 1.0),
            smoothness: 0.0,
            metallic: 0.0,
        }
    }
}

