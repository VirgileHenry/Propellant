use crate::Transform;
use super::ObjectUniform;

#[repr(C)] // important for any data we send to the gpu
#[allow(unused)] // we don't use the fields directly, but they are used by the gpu
#[derive(Debug, Clone, Copy)]
pub struct ModelMatrixUniformObject {
    pub model: glam::Mat4,
}

impl ObjectUniform for ModelMatrixUniformObject {
    type FromComponent = Transform;
    fn set_uniform(transform: &Self::FromComponent, write_to_buf: &mut dyn FnMut(&[Self], usize), instance_count: usize) {
        for i in 0..instance_count {
            write_to_buf(&[ModelMatrixUniformObject {
                model: transform.world_pos(),
            }], i);
        }
    }
}