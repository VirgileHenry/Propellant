

/// This is the error type of the propellant engine.
/// Basically wraps other errors types into a single enum.
#[derive(Debug)]
pub enum PropellantError {
    VkError(vulkanalia::vk::ErrorCode),
    Residual(Box<dyn std::error::Error + Send + Sync + 'static>),
    LibLoadingError(String),
    NoFittingVulkanDevice,
    NoVulkanInterface,
    OutOfMemory,
    EventLoopClosed,
}


impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for PropellantError {
    fn from(value: Box<dyn std::error::Error + Send + Sync + 'static>) -> PropellantError {
        PropellantError::Residual(value)
    }
}

impl From<vulkanalia::vk::ErrorCode> for PropellantError {
    fn from(value: vulkanalia::vk::ErrorCode) -> Self {
        PropellantError::VkError(value)
    }
}

impl<T> From<winit::event_loop::EventLoopClosed<T>> for PropellantError {
    fn from(_: winit::event_loop::EventLoopClosed<T>) -> Self {
        PropellantError::EventLoopClosed
    }
}