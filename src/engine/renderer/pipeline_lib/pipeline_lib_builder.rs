use std::collections::HashMap;

use crate::{engine::{errors::{PropellantError, PResult}, renderer::rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder}, id};

use super::GraphicPipelineLib;



#[derive(Debug)]
pub struct GraphicPipelineLibBuilder {
    lib: HashMap<u64, RenderingPipelineBuilder>,
}

impl GraphicPipelineLibBuilder {
    pub fn build(
        &self,
        vk_device: &vulkanalia::Device,
        swapchain_extent: vulkanalia::vk::Extent2D,
        swapchain_images: &[vulkanalia::vk::Image],
        render_pass: vulkanalia::vk::RenderPass
    ) -> PResult<GraphicPipelineLib> {
        Ok(GraphicPipelineLib::new(
            self.lib
                .iter()
                .map(|(k, v)|
                    v.build(vk_device, swapchain_extent, swapchain_images, render_pass).map(|p| (*k, p))
                ).collect::<Result<HashMap<_, _>, PropellantError>>()?
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