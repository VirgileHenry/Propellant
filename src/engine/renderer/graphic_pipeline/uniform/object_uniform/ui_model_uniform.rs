use crate::UiTransform;
use super::ObjectUniform;

#[repr(C)] // important for any data we send to the gpu
#[allow(unused)] // we don't use the fields directly, but they are used by the gpu
#[derive(Debug, Clone, Copy)]
pub struct UiPosUniformObject {
    pub position: glam::Vec2,
    pub relative_position: glam::Vec2,
    pub size: glam::Vec2,
    pub relative_size: glam::Vec2,
    pub anchor: glam::Vec2,
}

impl ObjectUniform for UiPosUniformObject {
    type FromComponent = UiTransform;
    fn get_uniform(transform: &UiTransform) -> Self {
        transform.to_uniform()
    }
}