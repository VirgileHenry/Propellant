use std::fmt::Display;

use image::ImageError;


/// Error while trying to load ressources for the engine.
#[derive(Debug)]
pub enum LoadingError {
    /// Unable to load the vulkan library.
    /// This is a fatal error, and the engine will not be able to run.
    VulkanLibrary(String),
    /// Error while loading a texture.
    Texture(ImageError),
}

impl Display for LoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadingError::VulkanLibrary(e) => write!(f, "Unable to load Vulkan library: {}", e),
            LoadingError::Texture(e) => write!(f, "Texture error: {}", e),
        }
    }
}