use std::fmt::Debug;

use foundry::ComponentTable;

pub(crate) mod camera_uniform;
pub(crate) mod main_directionnal_light;
#[cfg(feature = "ui")]
pub(crate) mod ui_resolution;

/// handle around a per frame uniform
/// It acts as the layer between our raw uniform buffer and a more abstract uniform object.
pub trait FrameUniform: Debug {
    /// Get the uniform to send to the gpu for the components
    fn get_uniform(components: &ComponentTable) -> Self;
}

