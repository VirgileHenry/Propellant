use crate::{
    Transform,
    Camera,
};

use super::FrameUniform;

#[repr(C)] // important for any data we send to the gpu
#[allow(unused)] // we don't use the fields directly, but they are used by the gpu
#[derive(Debug, Clone, Copy)]
pub struct CameraUniformObject {
    proj: glam::Mat4,
    view: glam::Mat4,
}

impl FrameUniform for CameraUniformObject {
    fn get_uniform(components: &foundry::ComponentTable) -> Self {
        for (_, tf, cam) in components.query2d::<Transform, Camera>() {
            if cam.is_main() {
                return CameraUniformObject {
                    proj: cam.projection_matrix(),
                    view:  tf.world_pos(),
                };
            }
        }

        CameraUniformObject {
            proj: glam::Mat4::ZERO,
            view: glam::Mat4::ZERO,
        }
    }
}