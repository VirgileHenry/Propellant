use crate::UiTransform;
use super::ObjectUniform;

#[repr(C)] // important for any data we send to the gpu
#[allow(unused)] // we don't use the fields directly, but they are used by the gpu
#[derive(Debug, Clone, Copy)]
pub struct UiPosUniformObject {
    pub pos: glam::Mat3,
}

impl Default for UiPosUniformObject {
    fn default() -> Self {
        UiPosUniformObject {
            pos: glam::Mat3::from_translation(glam::Vec2::new(0.0, 0.0))
        }
    }
}

impl ObjectUniform for UiPosUniformObject {
    type FromComponent = UiTransform;
    fn get_uniform(transform: &UiTransform) -> Self {
        transform.get_pos()
    }
}