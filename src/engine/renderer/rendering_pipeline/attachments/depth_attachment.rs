use crate::engine::{
    errors::{
        PResult,
        PropellantError,
        rendering_error::RenderingError
    },
    window::vulkan::{
        vulkan_image::{
            VulkanImage,
            vulkan_image_view::create_image_view
        },
    }
};

use vulkanalia::vk::InstanceV1_0;


pub fn create_depth_objects(
    vk_instance: &vulkanalia::Instance,
    vk_device: &vulkanalia::Device,
    vk_physical_device: vulkanalia::vk::PhysicalDevice,
    swapchain_extent: vulkanalia::vk::Extent2D,
) -> PResult<(VulkanImage, vulkanalia::vk::ImageView)> {

    let format = get_depth_format(vk_instance, vk_physical_device)?;

    let image = VulkanImage::create(
        vk_instance,
        vk_device,
        vk_physical_device,
        swapchain_extent.width,
        swapchain_extent.height,
        vulkanalia::vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        format
    )?;

    let image_view = create_image_view(
        vk_device,
        &image,
        format,
        vulkanalia::vk::ImageAspectFlags::DEPTH,
    )?;

    // record a command buffer to transition the depth image to a depth attachment
    /*
    transfer_manager.register_transition_image_layout(
        vk_device,
        image.image(),
        format,
        vulkanalia::vk::ImageLayout::UNDEFINED,
        vulkanalia::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
    )?;
    */

    Ok((image, image_view))
}

fn get_supported_format(
    vk_instance: &vulkanalia::Instance,
    vk_physical_device: vulkanalia::vk::PhysicalDevice,
    candidates: &[vulkanalia::vk::Format],
    tiling: vulkanalia::vk::ImageTiling,
    features: vulkanalia::vk::FormatFeatureFlags,
) -> PResult<vulkanalia::vk::Format> {
    candidates
        .iter()
        .cloned()
        .find(|f| {
            let properties = unsafe {vk_instance.get_physical_device_format_properties(
                vk_physical_device,
                *f,
            ) };

            match tiling {
                vulkanalia::vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                vulkanalia::vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                _ => false,
            }
        })
        .ok_or(PropellantError::Rendering(RenderingError::NoSupportedDepthFormat))
}

pub fn get_depth_format(
    vk_instance: &vulkanalia::Instance,
    vk_physical_device: vulkanalia::vk::PhysicalDevice
) -> PResult<vulkanalia::vk::Format> {
    let candidates = &[
        vulkanalia::vk::Format::D32_SFLOAT,
        vulkanalia::vk::Format::D32_SFLOAT_S8_UINT,
        vulkanalia::vk::Format::D24_UNORM_S8_UINT,
    ];

    get_supported_format(
        vk_instance,
        vk_physical_device,
        candidates,
        vulkanalia::vk::ImageTiling::OPTIMAL,
        vulkanalia::vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}