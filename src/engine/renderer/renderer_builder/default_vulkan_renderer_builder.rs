use crate::{engine::{errors::PResult, renderer::{VulkanRenderer, DefaultVulkanRenderer}}, VulkanInterface};
use crate::engine::renderer::rendering_pipeline::rendering_pipeline_builder::rendering_pipeline_builder_states::RPBSReady;
use crate::engine::renderer::rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder;

use super::VulkanRendererBuilder;



pub struct DefaultVulkanRendererBuilder {
    rendering_pipeline: RenderingPipelineBuilder<RPBSReady>
}

impl DefaultVulkanRendererBuilder {
    pub fn default() -> Box<DefaultVulkanRendererBuilder> {
        Box::new(DefaultVulkanRendererBuilder {
            rendering_pipeline: RenderingPipelineBuilder::default()
        })
    }

    pub fn with_pipeline(self: Box<Self>, pipeline: RenderingPipelineBuilder<RPBSReady>) -> Box<DefaultVulkanRendererBuilder> {
        Box::new(DefaultVulkanRendererBuilder {
            rendering_pipeline: pipeline,
            ..*self
        })
    }
}

impl VulkanRendererBuilder for DefaultVulkanRendererBuilder {
    fn build(
            self: Box<Self>,
            vk_interface: &mut VulkanInterface,
            window: &winit::window::Window,
        ) -> PResult<Box<dyn VulkanRenderer>> {
        Ok(Box::new(DefaultVulkanRenderer::new(vk_interface, window, self.rendering_pipeline)?))
    }

}
