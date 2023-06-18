use image::ImageError;



#[derive(Debug)]
pub enum LoadingError {
    VulkanLibrary(String),
    Texture(ImageError),
}