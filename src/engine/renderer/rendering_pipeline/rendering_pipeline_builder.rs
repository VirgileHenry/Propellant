
use crate::{
    engine::{
        errors::{
            PResult
        },
        window::vulkan::queues::QueueFamilyIndices,
        renderer::graphics_pipeline::graphics_pipeline_builder::GraphicsPipelineBuilder, consts::PROPELLANT_DEBUG_FEATURES
    }, 
    id
};

use self::{rendering_pipeline_layer::RenderingPipelineLayer, rendering_pipeline_builder_states::{RenderingPipelineBuilderStateWaitingPipelineLayer, RenderingPipelineBuilderStateWaitingFramebufferLayer, RenderingPipelineBuilderStateReady}, intermediate_render_target_builder::IntermediateRenderTargetBuilder};

use super::RenderingPipeline;

pub(crate) mod rendering_pipeline_builder_states;
pub(crate) mod rendering_pipeline_layer;
pub(crate) mod intermediate_render_target_builder;

pub struct RenderingPipelineBuilder<T> {
    phantom: std::marker::PhantomData<T>,
    pipelines: Vec<RenderingPipelineLayer>,
    framebuffers: Vec<IntermediateRenderTargetBuilder>,
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

    pub fn with_framebuffer_layer(self, layer: IntermediateRenderTargetBuilder) -> Self {
        let mut layers = self.framebuffers;
        layers.push(layer);

        RenderingPipelineBuilder {
            phantom: std::marker::PhantomData,
            pipelines: self.pipelines,
            framebuffers: layers,
        }
    }

    pub fn finish(self) -> RenderingPipelineBuilder<RenderingPipelineBuilderStateReady> {
        // check the layout of the pipeline is correct
        if PROPELLANT_DEBUG_FEATURES {
            assert!(self.pipelines.len() > 0, "[PROPELLANT DEBUG] Tried to create a rendering pipeline with no layers.");
            assert!(self.pipelines.len() == self.framebuffers.len() + 1, "[PROPELLANT DEBUG] Tried to create a rendering pipeline with an incorrect number of layers / intermediate render targets.");
        }

        RenderingPipelineBuilder {
            phantom: std::marker::PhantomData,
            pipelines: self.pipelines,
            framebuffers: self.framebuffers,
        }
    }
}

impl RenderingPipelineBuilder<RenderingPipelineBuilderStateReady> {
    pub fn build(
        self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        window: &winit::window::Window,
        surface: vulkanalia::vk::SurfaceKHR,
        queue_indices: QueueFamilyIndices,
    ) -> PResult<RenderingPipeline> {
        RenderingPipeline::create(
            self,
            vk_instance,
            window,
            surface,
            vk_device,
            vk_physical_device,
            queue_indices,
        )
    }

    pub fn layers(self) -> (impl Iterator<Item = (RenderingPipelineLayer, IntermediateRenderTargetBuilder)>, RenderingPipelineLayer) {
        let transition_layer_count = self.pipelines.len() - 1;
        let mut layers_iter = self.pipelines.into_iter();
        let mut transtion_pipelines = Vec::with_capacity(transition_layer_count);
        for _ in 0..transition_layer_count {
            transtion_pipelines.push(layers_iter.next().unwrap());
        }
        let transition_layers = transtion_pipelines.into_iter().zip(self.framebuffers.into_iter());
        let final_layer = layers_iter.next().unwrap();
        (transition_layers, final_layer)
    }

}



impl Default for RenderingPipelineBuilder<RenderingPipelineBuilderStateReady> {
    fn default() -> Self {
        RenderingPipelineBuilder::new().with_pipeline_layer(
            RenderingPipelineLayer::new().with_pipeline(id("default"), GraphicsPipelineBuilder::default())
        ).finish()
    }
}