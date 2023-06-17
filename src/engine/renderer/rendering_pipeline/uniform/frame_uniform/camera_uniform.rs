use foundry::component_iterator;

use crate::{Transform, Camera, engine::errors::{PResult, PropellantError, rendering_error::RenderingError}};

use super::AsPerFrameUniform;


#[repr(C)] // important for any data we send to the gpu
#[allow(unused)] // we don't use the fields directly, but they are used by the gpu
#[derive(Debug, Clone, Copy)]
pub struct CameraUniformObject {
    proj: glam::Mat4,
    view: glam::Mat4,
}

impl AsPerFrameUniform for CameraUniformObject {
    fn get_uniform(components: &foundry::ComponentTable) -> PResult<Self> {
        for (_, (tf, cam)) in component_iterator!(components; mut Transform, Camera) {
            if cam.is_main() {
                return Ok(CameraUniformObject {
                    proj: cam.projection_matrix(),
                    view:  tf.world_pos(),
                });
            }
        }

        Err(PropellantError::Rendering(RenderingError::NoMainCamera))
    }
}