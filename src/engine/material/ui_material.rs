use foundry::AsAny;

use crate::ColoredTexture;


#[allow(unused)]
#[repr(C)]
#[derive(Debug, Clone, Copy, AsAny)]
pub struct UiMaterial {
    pub texture: ColoredTexture,
    pub corner_radius: f32,
    /// experimentally correct
    padd: [f32; 3],
}

impl UiMaterial {
    pub fn new(texture: ColoredTexture, corner_radius: f32) -> UiMaterial {
        UiMaterial {
            texture,
            corner_radius,
            padd: [0.0; 3],
        }
    }

    pub fn colored(color: glam::Vec3, corner_radius: f32) -> UiMaterial {
        UiMaterial {
            texture: ColoredTexture::color(color),
            corner_radius,
            padd: [0.0; 3],
        }
    }
}