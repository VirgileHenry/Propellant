use std::fmt::Display;

use image::ImageError;



#[derive(Debug)]
pub enum LoadingError {
    VulkanLibrary(String),
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