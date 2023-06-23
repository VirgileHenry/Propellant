use crate::engine::errors::PResult;

use super::VulkanImage;

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;


pub fn create_image_view(
    vk_device: &vulkanalia::Device,
    image: &VulkanImage,
    format: vulkanalia::vk::Format,
    aspects: vulkanalia::vk::ImageAspectFlags,
) -> PResult<vulkanalia::vk::ImageView> {
    let subresource_range = vulkanalia::vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let info = vulkanalia::vk::ImageViewCreateInfo::builder()
        .image(image.image())
        .view_type(vulkanalia::vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(unsafe { vk_device.create_image_view(&info, None)? })
}