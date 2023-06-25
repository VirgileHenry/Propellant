use crate::{engine::errors::PResult, VulkanInterface};

use super::VulkanRenderer;

pub(crate) mod default_vulkan_renderer_builder;

pub trait VulkanRendererBuilder {
    fn build(
        self: Box<Self>,
        vk_interface: &mut VulkanInterface,
        window: &winit::window::Window,
    ) -> PResult<Box<dyn VulkanRenderer>>;
}