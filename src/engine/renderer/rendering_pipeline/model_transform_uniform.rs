use crate::{Transform, engine::errors::PResult};

/// Uniform object for model matrix.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ModelTransformObject {
    world_pos: glam::Mat4,
}

impl ModelTransformObject {
    pub fn new(transform: &Transform) -> ModelTransformObject {
        let world_pos = transform.world_pos();
        ModelTransformObject { world_pos }
    }
}

pub fn transform_uniform_generator(tf: &Transform) -> PResult<ModelTransformObject> {
    Ok(ModelTransformObject::new(tf))
}