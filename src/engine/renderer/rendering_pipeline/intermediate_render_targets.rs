use crate::engine::{
    window::vulkan::vulkan_image::VulkanImage,
    errors::PResult
};

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;


pub struct IntermediateRenderTargetBuilder {
    render_textures_formats: Vec<(vulkanalia::vk::Format, vulkanalia::vk::ImageAspectFlags)>,
}

impl IntermediateRenderTargetBuilder {
    pub fn new() -> Self {
        IntermediateRenderTargetBuilder {
            render_textures_formats: Vec::new(),
        }
    }

    pub fn add_render_texture(&mut self, format: vulkanalia::vk::Format, aspect: vulkanalia::vk::ImageAspectFlags) {
        self.render_textures_formats.push((format, aspect));
    }

    pub fn build(
        self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        render_pass: vulkanalia::vk::RenderPass,
        width: u32,
        height: u32,
    ) -> PResult<IntermediateRenderTarget> {
        let (images, views): (Vec<VulkanImage>, Vec<vulkanalia::vk::ImageView>) = self.render_textures_formats.into_iter().map(|(format, aspects)| {
            Self::create_image_and_view(vk_instance, vk_device, vk_physical_device, width, height, format, aspects)
        }).collect::<PResult<Vec<_>>>()?.into_iter().unzip();

        let create_info = vulkanalia::vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(&views)
            .width(width)
            .height(height)
            .layers(1);
        let framebuffer = unsafe {
            vk_device.create_framebuffer(&create_info, None)?
        };

        Ok(IntermediateRenderTarget {
            images,
            views,
            framebuffer,
        })
    }

    fn create_image_and_view(
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        width: u32,
        height: u32,
        format: vulkanalia::vk::Format,
        aspects: vulkanalia::vk::ImageAspectFlags,
    ) -> PResult<(VulkanImage, vulkanalia::vk::ImageView)> {
        let image = VulkanImage::create(
            vk_instance,
            vk_device,
            vk_physical_device,
            width,
            height,
            vulkanalia::vk::ImageUsageFlags::COLOR_ATTACHMENT,
            format,
        )?;

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

        let view = unsafe { vk_device.create_image_view(&info, None)? };

        Ok((image, view))
    }
}



pub struct IntermediateRenderTarget {
    images: Vec<VulkanImage>,
    views: Vec<vulkanalia::vk::ImageView>,
    framebuffer: vulkanalia::vk::Framebuffer,
}

impl IntermediateRenderTarget {
    pub fn new(
        images: Vec<VulkanImage>,
        views: Vec<vulkanalia::vk::ImageView>,
        framebuffer: vulkanalia::vk::Framebuffer
    ) -> Self {
        IntermediateRenderTarget {
            images,
            views,
            framebuffer,
        }
    }

    pub fn framebuffer(&self) -> vulkanalia::vk::Framebuffer {
        self.framebuffer
    }

    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.images.iter_mut().for_each(|image| image.destroy(vk_device));
        self.views.iter_mut().for_each(|view| unsafe { vk_device.destroy_image_view(*view, None) });
        unsafe { vk_device.destroy_framebuffer(self.framebuffer, None) }
    }
}