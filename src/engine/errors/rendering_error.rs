use std::fmt::Display;

/// Errors that occured during rendering.
#[derive(Debug)]
pub enum RenderingError {
    /// Any vulkan error.
    Vulkan(vulkanalia::vk::ErrorCode),
    /// There is no vulkan device that can be used for rendering.
    /// This mean that no GPU on the device was compatible with the engine.
    NoFittingVulkanDevice,
    /// There is no vulkan interface, so we can't do any vulkan calls.
    NoVulkanInterface,
    /// The entity position in a uniform buffer is not known.
    /// This entity can't set it's transforms to the shaders.
    /// This may be caused by a scene that have not been properly rebuilt.
    UnknownEntityBufferPosition,
    /// The material cast is invalid.
    /// This comes from a try to send a material to the shader that was not in the expected type.
    InvalidMaterialCast,
}

impl From<vulkanalia::vk::ErrorCode> for RenderingError {
    fn from(value: vulkanalia::vk::ErrorCode) -> Self {
        RenderingError::Vulkan(value)
    }
}

impl Display for RenderingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderingError::Vulkan(e) => write!(f, "Vulkan error: {}", e),
            RenderingError::NoFittingVulkanDevice => write!(f, "No fitting vulkan device."),
            RenderingError::NoVulkanInterface => write!(f, "No vulkan interface."),
            RenderingError::UnknownEntityBufferPosition => write!(f, "Unknown entity buffer position. The scene have not been properly rebuilt."),
            RenderingError::InvalidMaterialCast => write!(f, "Invalid material cast."),
        }
    }
}