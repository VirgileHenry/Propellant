use std::fmt::Display;


#[derive(Debug)]
pub enum RenderingError {
    Vulkan(vulkanalia::vk::ErrorCode),
    NoFittingVulkanDevice,
    NoVulkanInterface,
    UnknownEntityBufferPosition,
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