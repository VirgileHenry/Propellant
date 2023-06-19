use std::fmt::Display;

/// Debug Errors.
/// Errors that will only exists in debug builds, but will throw helpfull results of wrong uses of the images.
pub enum DebugError {
    /// The vulkan debug layers are not present, and therfore there is no way to debug the vulkan calls.
    MissingVulkanDebugLayers,
    /// There is no resources library in the ecs, so nothing to render.
    MissingResourceLibrary,
    /// There is no main camera in the scene, so we can't render.
    NoMainCamera,
    /// There is no main light in the scene. This is not a fatal error, but it is a warning.
    NoMainLight,
}

impl Display for DebugError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugError::MissingVulkanDebugLayers => write!(f, "Missing Vulkan debug layers"),
            DebugError::MissingResourceLibrary => write!(f, "Missing resource library"),
            DebugError::NoMainCamera => write!(f, "No main camera"),
            DebugError::NoMainLight => write!(f, "No main light"),
        }
    }
}