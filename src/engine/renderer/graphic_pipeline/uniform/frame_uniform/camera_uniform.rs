use foundry::ComponentTable;

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
    fn set_uniform(components: &ComponentTable, write_to_buf: &mut dyn FnMut(&[Self])) {
        // todo : we could write the raw matrix here ?
        for (_, tf, cam) in components.query2d::<Transform, Camera>() {
            if cam.is_main() {
                write_to_buf(&[CameraUniformObject {
                    proj: cam.projection_matrix(),
                    view:  tf.world_pos(),
                }]);
                return;
            }
        }

        write_to_buf(&[CameraUniformObject {
            proj: glam::Mat4::ZERO,
            view: glam::Mat4::ZERO,
        }]);
    }
}