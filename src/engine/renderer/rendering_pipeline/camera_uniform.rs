use crate::{Transform, Camera};

// let's start with the uniforms.
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
