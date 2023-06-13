
#[derive(Debug)]
pub enum RenderingError {
    Vulkan(vulkanalia::vk::ErrorCode),
    NoMainCamera,
    NoFittingVulkanDevice,
    NoVulkanInterface,
    UnknownEntityBufferPosition,
}

impl From<vulkanalia::vk::ErrorCode> for RenderingError {
    fn from(value: vulkanalia::vk::ErrorCode) -> Self {
        RenderingError::Vulkan(value)
    }
}