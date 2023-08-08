use crate::{engine::renderer::graphic_pipeline::graphic_pipeline_builder::GraphicPipelineBuilderInterface, IntermediateRenderTargetBuilder, FinalRenderTargetBuilder};


/// The rendering pipeline is currently registering graphic pipelines.
pub struct RPBSRegisteringGraphic {
    pub graphic_pipelines: Vec<(u64, Box<dyn GraphicPipelineBuilderInterface>)>,
}

/// The rendering pipeline is currently waiting for compute pipelines.
pub struct RPBSWaitingComputePipeline {
    pub graphic_pipelines: Vec<(u64, Box<dyn GraphicPipelineBuilderInterface>)>,
    pub compute_pipelines: Vec<(u64, IntermediateRenderTargetBuilder, Box<(/* compute pipeline builder */)>)>,
    pub intermediate_rt: IntermediateRenderTargetBuilder,
}

/// The rendering pipeline is currently waiting for render targets.
pub struct RPBSWaitingRenderTargets {
    pub graphic_pipelines: Vec<(u64, Box<dyn GraphicPipelineBuilderInterface>)>,
    pub compute_pipelines: Vec<(u64, IntermediateRenderTargetBuilder, Box<(/* compute pipeline builder */)>)>,
}

/// The pipeline is ready to be built.
pub struct RPBSReady {
    pub graphic_pipelines: Vec<(u64, Box<dyn GraphicPipelineBuilderInterface>)>,
    pub compute_pipelines: Vec<(u64, IntermediateRenderTargetBuilder, Box<(/* compute pipeline builder */)>)>,
    pub final_rt: FinalRenderTargetBuilder,
}

