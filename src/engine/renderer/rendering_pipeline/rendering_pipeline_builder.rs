use std::collections::HashMap;

use crate::{
    engine::{
        errors::{
            PropellantError,
            PResult
        },
        window::vulkan::transfer_command_manager::TransferCommandManager,
        renderer::graphics_pipeline::graphics_pipeline_builder::GraphicsPipelineBuilder
    }, 
    id
};

use self::{rendering_pipeline_layer::RenderingPipelineLayer, rendering_pipeline_builder_states::{RenderingPipelineBuilderStateWaitingPipelineLayer, RenderingPipelineBuilderStateWaitingFramebufferLayer, RenderingPipelineBuilderStateReady}};

use super::RenderingPipeline;

pub(crate) mod rendering_pipeline_builder_states;
pub(crate) mod rendering_pipeline_layer;

pub struct RenderingPipelineBuilder<T> {
    phantom: std::marker::PhantomData<T>,
    pipelines: Vec<RenderingPipelineLayer>,
    framebuffers: Vec<()>,
}

impl RenderingPipelineBuilder<RenderingPipelineBuilderStateWaitingPipelineLayer> {
    pub fn new() -> Self {
        RenderingPipelineBuilder {
            phantom: std::marker::PhantomData,
            pipelines: Vec::new(),
            framebuffers: Vec::new(),
        }
    }

    pub fn with_pipeline_layer(self, layer: RenderingPipelineLayer) -> RenderingPipelineBuilder<RenderingPipelineBuilderStateWaitingFramebufferLayer> {
        let mut layers = self.pipelines;
        layers.push(layer);

        RenderingPipelineBuilder {
            phantom: std::marker::PhantomData,
            pipelines: layers,
            framebuffers: self.framebuffers,
        }
    }
}

impl RenderingPipelineBuilder<RenderingPipelineBuilderStateWaitingFramebufferLayer> {

    pub fn with_framebuffer_layer(self, layer: ()) -> Self {
        let mut layers = self.framebuffers;
        layers.push(layer);

        RenderingPipelineBuilder {
            phantom: std::marker::PhantomData,
            pipelines: self.pipelines,
            framebuffers: layers,
        }
    }

    pub fn finish(self) -> RenderingPipelineBuilder<RenderingPipelineBuilderStateReady> {
        RenderingPipelineBuilder {
            phantom: std::marker::PhantomData,
            pipelines: self.pipelines,
            framebuffers: self.framebuffers,
        }
    }
}

impl RenderingPipelineBuilder<RenderingPipelineBuilderStateReady> {
    pub fn build(
        &self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        swapchain_extent: vulkanalia::vk::Extent2D,
        swapchain_images: &[vulkanalia::vk::Image],
        render_pass: vulkanalia::vk::RenderPass,
        transfer_manager: &mut TransferCommandManager,
    ) -> PResult<RenderingPipeline> {
        Ok(RenderingPipeline::new(
            self.pipelines[0]
                .iter()
                .map(|(k, v)|
                    v.build(
                        vk_instance,
                        vk_device,
                        vk_physical_device,
                        transfer_manager,
                        swapchain_extent,
                        swapchain_images,
                        render_pass,
                    ).map(|p| (k, p))
                ).collect::<Result<HashMap<_, _>, PropellantError>>()?
        ))
    }
}



impl Default for RenderingPipelineBuilder<RenderingPipelineBuilderStateReady> {
    fn default() -> Self {
        RenderingPipelineBuilder::new().with_pipeline_layer(
            RenderingPipelineLayer::new().with_pipeline(id("default"), GraphicsPipelineBuilder::default())
        ).finish()
    }
}