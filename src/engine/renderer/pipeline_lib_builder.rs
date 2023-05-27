use std::collections::HashMap;

use crate::{engine::errors::PropellantError, id};

use super::{rendering_pipeline_builder::RenderingPipelineBuilder, pipeline_lib::GraphicPipelineLib};



pub struct GraphicPipelineLibBuilder {
    lib: HashMap<u64, RenderingPipelineBuilder>,
}

impl GraphicPipelineLibBuilder {
    pub fn build(
        self,
        vk_device: &vulkanalia::Device,
        swapchain_extent: vulkanalia::vk::Extent2D,
        render_pass: vulkanalia::vk::RenderPass
    ) -> Result<GraphicPipelineLib, PropellantError> {
        Ok(GraphicPipelineLib::new(
            self.lib
                .into_iter()
                .map(|(k, v)| v.build(vk_device, swapchain_extent, render_pass).and_then(|p| Ok((k, p))))
                .collect::<Result<HashMap<_, _>, PropellantError>>()?
        ))
    }

    pub fn register_pipeline(&mut self, id: u64, pipeline: RenderingPipelineBuilder) {
        self.lib.insert(id, pipeline);
    }
}

impl Default for GraphicPipelineLibBuilder {
    fn default() -> Self {
        GraphicPipelineLibBuilder {
            lib: {
                let mut result = HashMap::new();
                result.insert(id("default"), RenderingPipelineBuilder::default());
                result
            },
        }
    }
}