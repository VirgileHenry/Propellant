
use std::collections::HashMap;

use crate::{
    engine::{
        errors::PResult,
        window::vulkan::queues::QueueFamilyIndices,
        renderer::graphics_pipeline::graphics_pipeline_builder::GraphicsPipelineBuilder,
    }, id, 
};

use self::{
    rendering_pipeline_builder_states::*,
};

use super::{RenderingPipeline, intermediate_render_targets::IntermediateRenderTargetBuilder, final_render_target::FinalRenderTargetBuilder};

pub(crate) mod rendering_pipeline_builder_states;
pub(crate) mod rendering_pipeline_layer;


pub struct RenderingPipelineBuilder<T> {
    state_data: T,
}

impl<T> RenderingPipelineBuilder<T> {
    pub fn state(&self) -> &T {
        &self.state_data
    }

    pub fn state_mut(&mut self) -> &mut T {
        &mut self.state_data
    }
}

impl RenderingPipelineBuilder<RPBSRegisteringGraphic> {
    pub fn new() -> Self {
        Self {
            state_data: RPBSRegisteringGraphic {
                graphic_pipelines: HashMap::new(),
            }
        }
    }

    pub fn with_graphic_pipeline(mut self, id: u64, pipeline: GraphicsPipelineBuilder) -> RenderingPipelineBuilder<RPBSRegisteringGraphic> {
        self.state_data.graphic_pipelines.insert(id, pipeline);
        self
    }

    pub fn with_intermediate_rt(self, render_texture: IntermediateRenderTargetBuilder) -> RenderingPipelineBuilder<RPBSWaitingComputePipeline> {
        RenderingPipelineBuilder {
            state_data: RPBSWaitingComputePipeline {
                graphic_pipelines: self.state_data.graphic_pipelines,
                compute_pipelines: Vec::new(),
                last_intermediate_rt: render_texture,
            }
        }
    }

    pub fn with_final_rt(self, render_target: FinalRenderTargetBuilder) -> RenderingPipelineBuilder<RPBSReady> {

        let new_state = RPBSReady {
            graphic_pipelines: self.state_data.graphic_pipelines,
            compute_pipelines: Vec::new(),
            final_render_target: render_target,
        };

        RenderingPipelineBuilder {
            state_data: new_state,
        }
    }
}

impl RenderingPipelineBuilder<RPBSWaitingComputePipeline> {

    pub fn with_compute_pipeline(self, pipeline: ()) -> RenderingPipelineBuilder<RPBSWaitingRenderTargets> {
        let mut previous_compute_pipelines = self.state_data.compute_pipelines;
        let new_compute_pipeline = (self.state_data.last_intermediate_rt, pipeline);
        previous_compute_pipelines.push(new_compute_pipeline);
        
        let new_state = RPBSWaitingRenderTargets {
            graphic_pipelines: self.state_data.graphic_pipelines,
            compute_pipelines: previous_compute_pipelines,
        };

        RenderingPipelineBuilder {
            state_data: new_state,
        }
    }
}

impl RenderingPipelineBuilder<RPBSWaitingRenderTargets> {
    pub fn with_intermediate_rt(self, render_target: IntermediateRenderTargetBuilder) -> RenderingPipelineBuilder<RPBSWaitingComputePipeline> {

        let new_state = RPBSWaitingComputePipeline {
            graphic_pipelines: self.state_data.graphic_pipelines,
            compute_pipelines: self.state_data.compute_pipelines,
            last_intermediate_rt: render_target,
        };

        RenderingPipelineBuilder {
            state_data: new_state,
        }
    }

    pub fn with_final_rt(self, final_render_target: FinalRenderTargetBuilder) -> RenderingPipelineBuilder<RPBSReady> {
        RenderingPipelineBuilder {
            state_data: RPBSReady {
                graphic_pipelines: self.state_data.graphic_pipelines,
                compute_pipelines: self.state_data.compute_pipelines,
                final_render_target,
            }
        }
    }
}

impl RenderingPipelineBuilder<RPBSReady> {
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



}

impl From<RenderingPipelineBuilder<RPBSReady>> for RPBSReady {
    fn from(builder: RenderingPipelineBuilder<RPBSReady>) -> Self {
        builder.state_data
    }
}

impl Default for RenderingPipelineBuilder<RPBSReady> {
    fn default() -> Self {
        RenderingPipelineBuilder::new()
            .with_graphic_pipeline(id("default"), GraphicsPipelineBuilder::default())
            .with_final_rt(FinalRenderTargetBuilder::default())
    }
}