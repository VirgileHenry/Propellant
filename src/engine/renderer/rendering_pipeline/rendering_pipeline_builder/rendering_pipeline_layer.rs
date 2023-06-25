use std::collections::HashMap;

use crate::engine::renderer::graphics_pipeline::graphics_pipeline_builder::GraphicsPipelineBuilder;



pub struct RenderingPipelineLayer{
    layer: HashMap<u64, GraphicsPipelineBuilder>,
}

impl RenderingPipelineLayer {
    pub fn new() -> Self {
        RenderingPipelineLayer {
            layer: HashMap::new(),
        }
    }

    pub fn with_pipeline(self, id: u64, pipeline: GraphicsPipelineBuilder) -> Self {
        let mut layer = self.layer;
        layer.insert(id, pipeline);
        RenderingPipelineLayer {
            layer,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (u64, &GraphicsPipelineBuilder)> {
        self.layer.iter().map(|(key, pipeline)| (*key, pipeline))
    }
}