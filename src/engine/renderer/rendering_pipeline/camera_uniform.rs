use foundry::{component_iterator, ComponentTable};

use crate::{Transform, Camera, engine::errors::{PropellantError, PResult}};

/// Uniform object for the camera, containing view and proj
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CameraUniformObject {
    proj_view: glam::Mat4,
}

impl CameraUniformObject {
    pub fn new(transform: &Transform, camera: &Camera) -> CameraUniformObject {
        let proj_view = camera.projection_matrix() * transform.world_pos();
        CameraUniformObject { proj_view }
    }
}


/// Functions that return a camera uniform object from the comp table. 
/// This is used to generate the camera uniform object for the rendering pipeline.
pub fn camera_uniform_generator(components: &ComponentTable) -> PResult<CameraUniformObject> {

    for (_, (tf, cam)) in component_iterator!(components; mut Transform, Camera) {
        if cam.is_main() {
            return Ok(CameraUniformObject::new(tf, cam));
        }
    }

    Err(PropellantError::NoMainCamera)
}