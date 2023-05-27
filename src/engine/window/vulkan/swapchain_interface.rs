use crate::engine::errors::PropellantError;
use super::queues::QueueFamilyIndices;
use super::swapchain_support::SwapchainSupport;

use std::ops::Deref;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;

/// Wraps up all the vulkan swapchain stuff.
pub struct SwapchainInterface {
    swapchain: vulkanalia::vk::SwapchainKHR,
    format: vulkanalia::vk::Format,
    extent: vulkanalia::vk::Extent2D,
    images: Vec<vulkanalia::vk::Image>,
    image_views: Vec<vulkanalia::vk::ImageView>,
}

impl SwapchainInterface {
    /// Creates a new swapchain interface.
    pub fn create(
        vk_instance: &vulkanalia::Instance,
        window: &winit::window::Window,
        surface: vulkanalia::vk::SurfaceKHR,
        physical_device: vulkanalia::vk::PhysicalDevice,
        device: &vulkanalia::Device,
        indices: QueueFamilyIndices,
    ) -> Result<SwapchainInterface, PropellantError> {
        let support = SwapchainSupport::get(vk_instance, physical_device, surface)?;

        let format = support.format();
        let present_mode = support.present_mode();
        let extent = support.extent(window);

        // let's try to get an image count at min image count + 1, if it is more than the max go at max
        // watch out, as 0 means illimited.
        let image_count = if 
            support.capabilities().max_image_count != 0 &&
            support.capabilities().min_image_count + 1 > support.capabilities().max_image_count
        {
            support.capabilities().max_image_count
        }
        else {
            support.capabilities().min_image_count + 1
        };

        // as we forced same queue for drawing and presentation, we can use exclusive sharing mode ?
        let image_sharing_mode = vulkanalia::vk::SharingMode::EXCLUSIVE;
        let queue_family_indices = vec![indices.index()]; // todo ? is this correct ?

        let info = vulkanalia::vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vulkanalia::vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(support.capabilities().current_transform)
            .composite_alpha(vulkanalia::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vulkanalia::vk::SwapchainKHR::null());

        let swapchain = unsafe {
            device.create_swapchain_khr(&info, None)?
        };

        // create the images and images views
        let images = unsafe {device.get_swapchain_images_khr(swapchain)?};
        let image_views = Self::create_image_views(device, &images, format.format)?;

        Ok(SwapchainInterface {
            swapchain,
            format: format.format,
            extent,
            images,
            image_views,
        })
    }

    /// Recreates the swapchain interface, when it gets invalidated.
    /// This will not destroy the old one. The caller must ensure that the old one is destroyed.
    pub fn recreate(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        window: &winit::window::Window,
        surface: vulkanalia::vk::SurfaceKHR,
        physical_device: vulkanalia::vk::PhysicalDevice,
        device: &vulkanalia::Device,
        indices: QueueFamilyIndices,
    ) -> Result<(), PropellantError> {
        // create the swapchain again.
        let support = SwapchainSupport::get(vk_instance, physical_device, surface)?;

        let format = support.format();
        let present_mode = support.present_mode();
        let extent = support.extent(window);

        // let's try to get an image count at min image count + 1, if it is more than the max go at max
        // watch out, as 0 means illimited.
        let image_count = if 
            support.capabilities().max_image_count != 0 &&
            support.capabilities().min_image_count + 1 > support.capabilities().max_image_count
        {
            support.capabilities().max_image_count
        }
        else {
            support.capabilities().min_image_count + 1
        };

        // as we forced same queue for drawing and presentation, we can use exclusive sharing mode ?
        let image_sharing_mode = vulkanalia::vk::SharingMode::EXCLUSIVE;
        let queue_family_indices = vec![indices.index()]; // todo ? is this correct ?

        let info = vulkanalia::vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(format.format)
            .image_color_space(format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vulkanalia::vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(support.capabilities().current_transform)
            .composite_alpha(vulkanalia::vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(self.swapchain);

        let swapchain = unsafe {
            device.create_swapchain_khr(&info, None)?
        };

        // create the images and images views
        let images = unsafe {device.get_swapchain_images_khr(swapchain)?};
        let image_views = Self::create_image_views(device, &images, format.format)?;

        // destroy everything
        unsafe {self.destroy(device);}
        
        // assign every new field.
        self.swapchain = swapchain;
        self.format = format.format;
        self.extent = extent;
        self.images = images;
        self.image_views = image_views;

        Ok(())
    }


    fn create_image_views(
        device: &vulkanalia::Device,
        swapchain_images: &Vec<vulkanalia::vk::Image>,
        swapchain_format: vulkanalia::vk::Format,
    ) -> Result<Vec<vulkanalia::vk::ImageView>, PropellantError> {
        Ok(swapchain_images.iter().map(|i| {
            let components = vulkanalia::vk::ComponentMapping::builder()
                .r(vulkanalia::vk::ComponentSwizzle::IDENTITY)
                .g(vulkanalia::vk::ComponentSwizzle::IDENTITY)
                .b(vulkanalia::vk::ComponentSwizzle::IDENTITY)
                .a(vulkanalia::vk::ComponentSwizzle::IDENTITY);
            let subresource_range = vulkanalia::vk::ImageSubresourceRange::builder()
                .aspect_mask(vulkanalia::vk::ImageAspectFlags::COLOR)
                .base_mip_level(0)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1);
            let info = vulkanalia::vk::ImageViewCreateInfo::builder()
                .image(*i)
                .view_type(vulkanalia::vk::ImageViewType::_2D)
                .format(swapchain_format)
                .components(components)
                .subresource_range(subresource_range);
                unsafe {device.create_image_view(&info, None)}
        }).collect::<Result<Vec<_>, _>>()?)
    }

    pub fn format(&self) -> vulkanalia::vk::Format {
        self.format
    }

    pub fn extent(&self) -> vulkanalia::vk::Extent2D {
        self.extent
    }

    pub fn images(&self) -> &Vec<vulkanalia::vk::Image> {
        &self.images
    }

    pub fn image_views(&self) -> &Vec<vulkanalia::vk::ImageView> {
        &self.image_views
    }
    
    /// Destroys the swapchain and all the image views.
    pub unsafe fn destroy(&self, device: &vulkanalia::Device) {
        self.image_views
            .iter()
            .for_each(|v| device.destroy_image_view(*v, None));
        device.destroy_swapchain_khr(self.swapchain, None);
    }
}

impl Deref for SwapchainInterface {
    type Target = vulkanalia::vk::SwapchainKHR;

    fn deref(&self) -> &Self::Target {
        // the underlying type is a u64,
        // so it's ok to return a copy of it as it's a reference
        &self.swapchain
    }
}