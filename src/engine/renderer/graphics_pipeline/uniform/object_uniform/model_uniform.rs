use super::AsPerObjectUniform;

#[repr(C)] // important for any data we send to the gpu
#[allow(unused)] // we don't use the fields directly, but they are used by the gpu
#[derive(Debug, Clone, Copy)]
pub struct ModelMatrixUniformObject {
    pub model: glam::Mat4,
}

impl AsPerObjectUniform for ModelMatrixUniformObject {
    fn get_uniform(transform: &crate::Transform, _material: &crate::Material) -> crate::engine::errors::PResult<Self> where Self: Sized {
        Ok(ModelMatrixUniformObject {
            model: transform.world_pos(),
        })
    }
}