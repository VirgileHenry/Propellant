use std::collections::HashMap;

use super::rendering_pipeline::RenderingPipeline;



pub struct GraphicPipelineLib {
    lib: HashMap<u64, RenderingPipeline>,
}

impl GraphicPipelineLib {
    pub fn new(lib: HashMap<u64, RenderingPipeline>) -> Self {
        GraphicPipelineLib { lib }
    }

    pub fn empty() -> Self {
        GraphicPipelineLib {
            lib: HashMap::new(),
        }
    }

    pub fn get_pipeline(&self, id: u64) -> Option<&RenderingPipeline> {
        self.lib.get(&id)
    }

    pub fn get_pipelines(&self) -> impl Iterator<Item = (u64, &RenderingPipeline)> {
        self.lib.iter().map(|(key, pipeline)| (*key, pipeline))
    }

    pub fn get_pipelines_mut(&mut self) -> impl Iterator<Item = (u64, &mut RenderingPipeline)> {
        self.lib.iter_mut().map(|(key, pipeline)| (*key, pipeline))
    }
}

