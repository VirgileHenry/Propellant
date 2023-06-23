use std::{error::Error, fmt::{Display, Debug}};

use image::ImageError;

use self::{
    rendering_error::RenderingError,
    loading_errors::LoadingError,
    debug_error::DebugError
};

pub(crate) mod rendering_error;
pub(crate) mod loading_errors;
pub(crate) mod debug_error;


/// The Propellant result.
pub type PResult<T> = Result<T, PropellantError>;

/// This is the error type of the propellant engine.
/// Basically wraps other errors types into a single enum.
pub enum PropellantError {
    Residual(Box<dyn std::error::Error + Send + Sync + 'static>),
    Loading(LoadingError),
    Rendering(RenderingError),
    DebugError(DebugError),
    OutOfMemory,
    EventLoopClosed,
    NoMainCamera,
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

impl From<LoadingError> for PropellantError {
    fn from(value: LoadingError) -> Self {
        PropellantError::Loading(value)
    }
}

impl From<vulkanalia::vk::ErrorCode> for PropellantError {
    fn from(value: vulkanalia::vk::ErrorCode) -> Self {
        PropellantError::Rendering(RenderingError::Vulkan(value))
    }
}

impl From<ImageError> for PropellantError {
    fn from(value: ImageError) -> Self {
        PropellantError::Loading(LoadingError::TextureCreation(value))
    }
}

impl Debug for PropellantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // debug / display for propellant error.
        // primary error info
        match self {
            PropellantError::DebugError(_) => write!(f, "[PROPELLANT DEBUG]"),
            _ => write!(f, "[PROPELLANT ERROR]"),
        }?;
        // secondary error info
        match self {
            PropellantError::Residual(e) => write!(f, " {}", e),
            PropellantError::Loading(e) => write!(f, " {}", e),
            PropellantError::Rendering(e) => write!(f, " {}", e),
            PropellantError::DebugError(e) => write!(f, " {}", e),
            PropellantError::OutOfMemory => write!(f, "Out of memory."),
            PropellantError::EventLoopClosed => write!(f, "Event loop closed."),
            PropellantError::NoMainCamera => write!(f, "No main camera."),
        }


    }
}

impl Display for PropellantError {
    /// Display the propellant errors. This is the same implementation as the debug, as errors
    /// are not meant to be displayed anywhere outside of a debug context anyway.
    /// However, this trait is needed to implement the error trait.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for PropellantError {

}