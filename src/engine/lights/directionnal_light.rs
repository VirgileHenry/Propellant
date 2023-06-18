
/// A directionnal light.
/// This has a alignment of 16, as it may be passed to shaders.
#[repr(C, align(16))]
#[derive(Debug, Clone)]
pub struct DirectionnalLight {
    pub direction: glam::Vec3,
    pub ambiant_color: glam::Vec3,
    pub diffuse_color: glam::Vec3,
}

impl DirectionnalLight {
    pub fn new(ambiant: glam::Vec3, diffuse: glam::Vec3, direction: glam::Vec3) -> Self {
        DirectionnalLight {
            direction,
            ambiant_color: ambiant,
            diffuse_color: diffuse,
        }
    }

    pub fn black() -> Self {
        DirectionnalLight {
            direction: glam::Vec3::NEG_Y,
            ambiant_color: glam::Vec3::ZERO,
            diffuse_color: glam::Vec3::ZERO,
        }
    }
}