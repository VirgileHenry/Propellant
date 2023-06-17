use super::AsPerObjectUniform;


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