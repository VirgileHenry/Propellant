
#[derive(Debug)]
pub enum RenderingError {
    Vulkan(vulkanalia::vk::ErrorCode),
    MissingMainCamera,
    MissingResources,
    MissingUniformData,
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