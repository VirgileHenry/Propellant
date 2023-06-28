use std::collections::HashMap;

use crate::{GraphicsPipelineBuilder, engine::renderer::rendering_pipeline::{intermediate_render_targets::IntermediateRenderTargetBuilder, final_render_target::FinalRenderTargetBuilder}};




pub struct RPBSRegisteringGraphic {
    pub graphic_pipelines: HashMap<u64, GraphicsPipelineBuilder>,
}
pub struct RPBSWaitingComputePipeline {
    pub graphic_pipelines: HashMap<u64, GraphicsPipelineBuilder>,
    pub compute_pipelines: Vec<(IntermediateRenderTargetBuilder, ())>,
    pub last_intermediate_rt: IntermediateRenderTargetBuilder,
}
pub struct RPBSWaitingRenderTargets {
    pub graphic_pipelines: HashMap<u64, GraphicsPipelineBuilder>,
    pub compute_pipelines: Vec<(IntermediateRenderTargetBuilder, ())>,
}

pub struct RPBSReady {
    pub graphic_pipelines: HashMap<u64, GraphicsPipelineBuilder>,
    pub compute_pipelines: Vec<(IntermediateRenderTargetBuilder, ())>,
    pub final_render_target: FinalRenderTargetBuilder,
}

