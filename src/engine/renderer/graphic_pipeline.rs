use crate::{
    engine::errors::PResult,
    PropellantResources
};

use self::graphic_pipeline_state::GraphicPipelineCreationState;


pub(crate) mod graphic_pipeline_builder;
pub(crate) mod graphic_pipeline_gen;
pub(crate) mod graphic_pipeline_state;
pub(crate) mod renderable_component;
pub(crate) mod uniform;

/// This trait is a handle around graphic pipelines.
/// Graphic pipelines are generic over the uniforms, so they can't be stored directly.
/// Instead, we hide them behind this trait, that provide access to all it's method.
pub trait GraphicPipelineInterface {
    fn recreation_cleanup(
        &mut self,
        vk_device: &vulkanalia::Device,
    );
    fn recreate(
        &mut self,
        vk_device: &vulkanalia::Device,
        swapchain_extent: vulkanalia::vk::Extent2D,
        render_pass: vulkanalia::vk::RenderPass,
    ) -> PResult<()>;
    fn register_draw_commands(
        &self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        command_buffer: vulkanalia::vk::CommandBuffer,
        resources: &PropellantResources,
    );
    fn update_uniform_buffers(
        &mut self,
        vk_device: &vulkanalia::Device,
        components: &foundry::ComponentTable,
        image_index: usize,
    ) -> PResult<()>;
    fn rebuild_rendering_map(
        &mut self,
        components: &foundry::ComponentTable,
    );
    fn assert_uniform_buffer_sizes(
        &mut self,
        image_index: usize,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
    ) -> PResult<()>;
    fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    );
}


/*
How to create pipelines: 

create_graphic_pipeline!(
    (VERTEX_SHADER_CODE, stage::Vertex),
    (FRAGMENT_SHADER_CODE, stage::Fragment);
    (FrameUniform uniforms::FrameUniform1 stage::Vertex), // set 0
    (FrameUniform uniforms::FrameUniform2 stage::Fragment), // set 1
    (ResourceUniform uniforms::FrameUniform3 stage::Fragment), // set 2
    (RenderableUniform uniforms::ObjectUniform1 stage::Fragment), // set 3
    (ObjectUniform uniforms::ObjectUniform2 stage::Vertex), // set4
);

this should geneate a builder and the pipeline, implementing the according traits for them.

*/
