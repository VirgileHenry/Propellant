
use crate::{engine::{
    errors::PResult,
    window::vulkan::queues::QueueFamilyIndices,
    renderer::graphic_pipeline::graphic_pipeline_builder::{GraphicPipelineBuilderInterface, default_phong_pipeline},
}, id};

use self::states::{RPBSRegisteringGraphic, RPBSWaitingComputePipeline, RPBSReady, RPBSWaitingRenderTargets};

use super::{
    RenderingPipeline,
    intermediate_render_targets::IntermediateRenderTargetBuilder,
    final_render_target::FinalRenderTargetBuilder
};

pub(crate) mod states;


pub struct RenderingPipelineBuilder<T> {
    // data relative to the current state
    state_data: T,
    // any state data
    clear_color: (f32, f32, f32),
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
                graphic_pipelines: Vec::new(),
            },
            clear_color: (0.0, 0.0, 0.0),
        }
    }

    pub fn with_graphic_pipeline<T: GraphicPipelineBuilderInterface + 'static>(mut self, id: u64, pipeline: T) -> RenderingPipelineBuilder<RPBSRegisteringGraphic> {
        self.state_data.graphic_pipelines.push((id, Box::new(pipeline)));
        self
    }

    pub fn with_intermediate_rt(self, intermediate_rt: IntermediateRenderTargetBuilder) -> RenderingPipelineBuilder<RPBSWaitingComputePipeline> {
        RenderingPipelineBuilder {
            state_data: RPBSWaitingComputePipeline {
                graphic_pipelines: self.state_data.graphic_pipelines,
                compute_pipelines: Vec::new(),
                intermediate_rt,
            },
            clear_color: self.clear_color,
        }
    }

    pub fn with_final_rt(self, final_rt: FinalRenderTargetBuilder) -> RenderingPipelineBuilder<RPBSReady> {

        let new_state = RPBSReady {
            graphic_pipelines: self.state_data.graphic_pipelines,
            compute_pipelines: Vec::with_capacity(0),
            final_rt,
        };

        RenderingPipelineBuilder {
            state_data: new_state,
            clear_color: self.clear_color,
        }
    }
}

impl RenderingPipelineBuilder<RPBSWaitingComputePipeline> {

    pub fn with_compute_pipeline(self, pipeline: (/* compute pipeline builder */), id: u64) -> RenderingPipelineBuilder<RPBSWaitingRenderTargets> {
        let mut previous_compute_pipelines = self.state_data.compute_pipelines;
        let new_compute_pipeline = (id, self.state_data.intermediate_rt, Box::new(pipeline));
        previous_compute_pipelines.push(new_compute_pipeline);
        
        let new_state = RPBSWaitingRenderTargets {
            graphic_pipelines: self.state_data.graphic_pipelines,
            compute_pipelines: previous_compute_pipelines,
        };

        RenderingPipelineBuilder {
            state_data: new_state,
            clear_color: self.clear_color,
        }
    }
}

impl RenderingPipelineBuilder<RPBSWaitingRenderTargets> {
    pub fn with_intermediate_rt(self, intermediate_rt: IntermediateRenderTargetBuilder) -> RenderingPipelineBuilder<RPBSWaitingComputePipeline> {

        let new_state = RPBSWaitingComputePipeline {
            graphic_pipelines: self.state_data.graphic_pipelines,
            compute_pipelines: self.state_data.compute_pipelines,
            intermediate_rt,
        };

        RenderingPipelineBuilder {
            state_data: new_state,
            clear_color: self.clear_color,
        }
    }

    pub fn with_final_rt(self, final_rt: FinalRenderTargetBuilder) -> RenderingPipelineBuilder<RPBSReady> {
        RenderingPipelineBuilder {
            state_data: RPBSReady {
                graphic_pipelines: self.state_data.graphic_pipelines,
                compute_pipelines: self.state_data.compute_pipelines,
                final_rt,
            },
            clear_color: self.clear_color,
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

impl<T> RenderingPipelineBuilder<T> {
    pub fn with_clear_color(self, clear_color: (f32, f32, f32)) -> RenderingPipelineBuilder<T> {
        RenderingPipelineBuilder {
            state_data: self.state_data,
            clear_color,
        }
    }

    pub fn clear_color(&self) -> (f32, f32, f32) {
        self.clear_color
    }
}

impl From<RenderingPipelineBuilder<RPBSReady>> for RPBSReady {
    fn from(builder: RenderingPipelineBuilder<RPBSReady>) -> Self {
        builder.state_data
    }
}

impl Default for RenderingPipelineBuilder<RPBSReady> {
    fn default() -> Self {
        // todo 
        if cfg!(feature = "ui") {
            RenderingPipelineBuilder::new()
                .with_graphic_pipeline(id("default"), default_phong_pipeline())
                .with_final_rt(FinalRenderTargetBuilder::default())
        }
        else {
            RenderingPipelineBuilder::new()
                .with_graphic_pipeline(id("default"), default_phong_pipeline())
                .with_final_rt(FinalRenderTargetBuilder::default())
        }
    }
}