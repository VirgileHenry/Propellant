use foundry::component_iterator;

use crate::{Transform, Camera, engine::errors::{PResult, PropellantError, rendering_error::RenderingError}};

use super::AsPerFrameUniform;


#[derive(Debug, Clone, Copy)]
pub struct CameraUniformObject {
    pub proj_view: glam::Mat4,
}

impl AsPerFrameUniform for CameraUniformObject {
    fn get_uniform(components: &foundry::ComponentTable) -> PResult<Self> {
        for (_, (tf, cam)) in component_iterator!(components; mut Transform, Camera) {
            if cam.is_main() {
                return Ok(CameraUniformObject {
                    proj_view: cam.projection_matrix() * tf.world_pos(),
                });
            }
        }

        Err(PropellantError::Rendering(RenderingError::NoMainCamera))
    }
}