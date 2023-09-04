use std::fmt::Debug;

use foundry::ComponentTable;

pub(crate) mod camera_uniform;
pub(crate) mod main_directionnal_light;

/// handle around a per frame uniform
/// It acts as the layer between our raw uniform buffer and a more abstract uniform object.
pub trait FrameUniform: Debug + Sized {
    /// Set the uniform to the gpu buffer.
    /// The write_to_buf function is a closure sending data to the buffer, and should be called with the according data.
    fn set_uniform(components: &ComponentTable, write_to_buf: &mut dyn FnMut(&[Self]));
}

