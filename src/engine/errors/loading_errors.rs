use std::fmt::Display;

use image::ImageError;

use crate::engine::mesh::loader::MeshLoadingError;


/// Error while trying to load ressources for the engine.
#[derive(Debug)]
pub enum LoadingError {
    /// Unable to load the vulkan library.
    /// This is a fatal error, and the engine will not be able to run.
    VulkanLibrary(String),
    /// Error while creating a texture from bytes.
    TextureCreation(ImageError),
    /// Error while transitionning a texture to a new layout.
    TextureLayoutTransitionMissing,
    /// Unable to load a mesh.
    MeshLoading(MeshLoadingError),
}

impl Display for LoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadingError::VulkanLibrary(e) => write!(f, "Unable to load Vulkan library: {}", e),
            LoadingError::TextureCreation(e) => write!(f, "Texture error: {}", e),
            LoadingError::TextureLayoutTransitionMissing => write!(f, "Texture layout transition missing"),
            LoadingError::MeshLoading(e) => write!(f, "Mesh loading error: {:?}", e),
        }
    }
}

impl From<MeshLoadingError> for LoadingError {
    fn from(value: MeshLoadingError) -> Self {
        LoadingError::MeshLoading(value)
    }
}