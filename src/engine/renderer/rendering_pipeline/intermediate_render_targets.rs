use crate::engine::{window::vulkan::vulkan_image::VulkanImage, errors::PResult};

use vulkanalia::vk::DeviceV1_0;

pub struct IntermediateRenderTarget {
    images: Vec<VulkanImage>,
    views: Vec<vulkanalia::vk::ImageView>,
    framebuffers: Vec<vulkanalia::vk::Framebuffer>,
}

impl IntermediateRenderTarget {
    pub fn new(
        vk_device: &vulkanalia::Device,
        count: usize,
    ) -> PResult<IntermediateRenderTarget> {


        todo!()
    }

    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.images.iter_mut().for_each(|image| image.destroy(vk_device));
        self.views.iter_mut().for_each(|view| unsafe { vk_device.destroy_image_view(*view, None) });
        self.framebuffers.iter_mut().for_each(|framebuffer| unsafe { vk_device.destroy_framebuffer(*framebuffer, None) });
    }
}