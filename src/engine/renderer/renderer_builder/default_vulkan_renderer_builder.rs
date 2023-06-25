use crate::{engine::{errors::PResult, renderer::{VulkanRenderer, DefaultVulkanRenderer}}, VulkanInterface};
use crate::engine::renderer::rendering_pipeline::rendering_pipeline_builder::rendering_pipeline_builder_states::RenderingPipelineBuilderStateReady;
use crate::engine::renderer::rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder;

use super::VulkanRendererBuilder;



pub struct DefaultVulkanRendererBuilder {
    rendering_pipeline: RenderingPipelineBuilder<RenderingPipelineBuilderStateReady>
}

impl DefaultVulkanRendererBuilder {
    pub fn default() -> Box<DefaultVulkanRendererBuilder> {
        Box::new(DefaultVulkanRendererBuilder {
            rendering_pipeline: RenderingPipelineBuilder::default()
        })
    }

    pub fn with_pipeline(self: Box<Self>, pipeline: RenderingPipelineBuilder<RenderingPipelineBuilderStateReady>) -> Box<DefaultVulkanRendererBuilder> {
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
        ) -> PResult<Box<dyn VulkanRenderer>> {
        Ok(Box::new(DefaultVulkanRenderer::new(vk_interface, self.rendering_pipeline)?))
    }

}
