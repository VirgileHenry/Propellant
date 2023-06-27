use std::collections::HashMap;

use crate::engine::renderer::graphics_pipeline::graphics_pipeline_builder::GraphicsPipelineBuilder;



pub struct RenderingPipelineLayer{
    pipelines: HashMap<u64, GraphicsPipelineBuilder>,
}

impl RenderingPipelineLayer {
    pub fn new() -> Self {
        RenderingPipelineLayer {
            pipelines: HashMap::new(),
        }
    }

    pub fn with_pipeline(self, id: u64, pipeline: GraphicsPipelineBuilder) -> Self {
        let mut layer = self.pipelines;
        layer.insert(id, pipeline);
        RenderingPipelineLayer {
            pipelines: layer,
        }
    }

    pub fn pipelines(&self) -> &HashMap<u64, GraphicsPipelineBuilder> {
        &self.pipelines
    }

    pub fn pipelines_mut(&mut self) -> &mut HashMap<u64, GraphicsPipelineBuilder> {
        &mut self.pipelines
    }
}