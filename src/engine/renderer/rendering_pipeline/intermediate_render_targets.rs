use crate::engine::{window::vulkan::vulkan_image::VulkanImage, errors::PResult};

use vulkanalia::vk::DeviceV1_0;

pub struct IntermediateRenderTarget {
    images: Vec<VulkanImage>,
    views: Vec<vulkanalia::vk::ImageView>,
    framebuffer: vulkanalia::vk::Framebuffer,
}

impl IntermediateRenderTarget {
    pub fn new(
        vk_device: &vulkanalia::Device,
    ) -> PResult<IntermediateRenderTarget> {
        unimplemented!()
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