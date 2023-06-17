use self::rendering_error::RenderingError;


/// The Propellant result.
pub type PResult<T> = Result<T, PropellantError>;

/// This is the error type of the propellant engine.
/// Basically wraps other errors types into a single enum.
#[derive(Debug)]
pub enum PropellantError {
    Residual(Box<dyn std::error::Error + Send + Sync + 'static>),
    LibLoading(String),
    MissingDebugInfo,
    OutOfMemory,
    EventLoopClosed,
    NoMainCamera,
    Rendering(RenderingError),
}


impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for PropellantError {
    fn from(value: Box<dyn std::error::Error + Send + Sync + 'static>) -> PropellantError {
        PropellantError::Residual(value)
    }
}

impl<T> From<winit::event_loop::EventLoopClosed<T>> for PropellantError {
    fn from(_: winit::event_loop::EventLoopClosed<T>) -> Self {
        PropellantError::EventLoopClosed
    }
}

impl From<RenderingError> for PropellantError {
    fn from(value: RenderingError) -> PropellantError {
        PropellantError::Rendering(value)
    }
}

impl From<vulkanalia::vk::ErrorCode> for PropellantError {
    fn from(value: vulkanalia::vk::ErrorCode) -> Self {
        PropellantError::Rendering(RenderingError::Vulkan(value))
    }
}

pub(crate) mod rendering_error;