use std::collections::HashMap;

use crate::{engine::errors::{PropellantError, PResult}, id};

use super::{rendering_pipeline_builder::RenderingPipelineBuilder, pipeline_lib::GraphicPipelineLib};


#[derive(Debug)]
pub struct GraphicPipelineLibBuilder {
    lib: HashMap<u64, RenderingPipelineBuilder>,
}

impl GraphicPipelineLibBuilder {
    pub fn build(
        &self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        swapchain_extent: vulkanalia::vk::Extent2D,
        swapchain_images: &[vulkanalia::vk::Image],
        render_pass: vulkanalia::vk::RenderPass
    ) -> PResult<GraphicPipelineLib> {
        Ok(GraphicPipelineLib::new(
            self.lib
                .iter()
                .map(|(k, v)|
                    v.build(vk_instance, vk_device, vk_physical_device, swapchain_extent, swapchain_images, render_pass).map(|p| (*k, p))
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