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

    pub fn get_pipelines(&self) -> impl Iterator<Item = &RenderingPipeline> {
        self.lib.values()
    }

    pub fn get_pipelines_mut(&mut self) -> impl Iterator<Item = &mut RenderingPipeline> {
        self.lib.values_mut()
    }
}

