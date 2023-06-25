use std::collections::HashMap;

use super::graphics_pipeline::GraphicsPipeline;

pub(crate) mod rendering_pipeline_builder;

pub struct RenderingPipeline {
    lib: HashMap<u64, GraphicsPipeline>,
}

impl RenderingPipeline {
    pub fn new(lib: HashMap<u64, GraphicsPipeline>) -> Self {
        RenderingPipeline { lib }
    }

    pub fn empty() -> Self {
        RenderingPipeline {
            lib: HashMap::new(),
        }
    }

    pub fn pipeline_count(&self) -> usize {
        self.lib.len()
    }

    pub fn get_pipeline(&self, id: u64) -> Option<&GraphicsPipeline> {
        self.lib.get(&id)
    }

    pub fn get_pipeline_mut(&mut self, id: u64) -> Option<&mut GraphicsPipeline> {
        self.lib.get_mut(&id)
    }

    pub fn get_pipelines(&self) -> impl Iterator<Item = (u64, &GraphicsPipeline)> {
        self.lib.iter().map(|(key, pipeline)| (*key, pipeline))
    }

    pub fn get_pipelines_mut(&mut self) -> impl Iterator<Item = (u64, &mut GraphicsPipeline)> {
        self.lib.iter_mut().map(|(key, pipeline)| (*key, pipeline))
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        for (_, mut pipeline) in self.lib.drain() {
            pipeline.destroy(vk_device);
        }
    }
}

