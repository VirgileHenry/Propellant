use std::collections::HashMap;

use crate::CameraUniformObject;
use crate::MainDirectionnalLight;
use crate::ModelMatrixUniformObject;
use crate::PhongMaterial;
use crate::create_graphic_pipeline;
use crate::engine::errors::PResult;
use crate::engine::mesh::vertex::Vertex;
use crate::engine::renderer::shaders::DEFAULT_FRAG;
use crate::engine::renderer::shaders::DEFAULT_VERT;
use crate::engine::renderer::shaders::UI_FRAG;
use crate::engine::renderer::shaders::UI_VERT;

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;

use super::GraphicPipelineInterface;
use super::renderable_component::RenderableComponent;
use super::uniform::frame_uniform::FrameUniform;
use super::uniform::object_uniform::ObjectUniform;
use super::uniform::uniform_buffer::UniformBufferBuilder;
use super::uniform::uniform_buffer::UniformBuffer;

pub trait GraphicPipelineBuilderInterface {
    fn build(
        self: Box<Self>,
        vk_device: &vulkanalia::Device,
        swapchain_extent: vulkanalia::vk::Extent2D,
        frame_count: usize,
        render_pass: vulkanalia::vk::RenderPass,
    ) -> PResult<Box<dyn GraphicPipelineInterface>>; 
}


fn create_shader_module(source_code: &[u32], vk_device: &vulkanalia::Device) -> PResult<vulkanalia::vk::ShaderModule> {
    let info = vulkanalia::vk::ShaderModuleCreateInfo::builder()
        .code_size(source_code.len() * 4)
        .code(source_code); // x4 because we are using u32, and length is in byte

    Ok(unsafe { vk_device.create_shader_module(&info, None)? })
}


fn create_descriptor_pool(
    vk_device: &vulkanalia::Device,
    descriptor_types: Vec<vulkanalia::vk::DescriptorType>,
    frame_count: usize,
) -> PResult<vulkanalia::vk::DescriptorPool> {

    let descriptor_set_count = descriptor_types.len() * frame_count;

    // for each layout type, we count how many descriptor sets we need.
    let mut ds_count_map = HashMap::with_capacity(3);
    
    for ds_type in descriptor_types.into_iter() {
        match ds_count_map.get_mut(&ds_type) {
            Some(count) => *count += frame_count,
            None => { ds_count_map.insert(ds_type, frame_count); },
        }
    }
    
    let pool_sizes = ds_count_map.into_iter().map(|(ds_type, count)| {
        vulkanalia::vk::DescriptorPoolSize::builder()
            .type_(ds_type)
            .descriptor_count(count as u32)
    }).collect::<Vec<_>>();

    let info = vulkanalia::vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(&pool_sizes)
        .max_sets(descriptor_set_count as u32);

    Ok( unsafe { 
        vk_device.create_descriptor_pool(&info, None)?
    })
}



pub fn default_phong_pipeline() -> impl GraphicPipelineBuilderInterface {
    use crate::ShaderStage;
    use crate::engine::renderer::graphic_pipeline::GraphicPipelineCreationState;
    create_graphic_pipeline!(
        (ShaderStage::Vertex, DEFAULT_VERT), // vert shader
        (ShaderStage::Fragment, DEFAULT_FRAG); // frag shader
        (FrameUniform, CameraUniformObject, ShaderStage::Vertex), // camera uniforms (one per frame)
        (FrameUniform, MainDirectionnalLight, ShaderStage::Fragment), // light uniforms (one per frame)
        (RenderableComponent, PhongMaterial, ShaderStage::Fragment), // phong material (one per object)
        (ObjectUniform, ModelMatrixUniformObject, ShaderStage::Vertex), // model matrix (one per object)
    )
}

#[cfg(feature = "ui")]
pub fn default_ui_pipeline() -> impl GraphicPipelineBuilderInterface {
    // create a new pipeline with the macro
    use crate::UiMaterial;
    use crate::engine::renderer::graphic_pipeline::uniform::object_uniform::ui_model_uniform::UiPosUniformObject;
    use crate::ShaderStage;
    use crate::engine::renderer::graphic_pipeline::GraphicPipelineCreationState;
    create_graphic_pipeline!(
        // provide the shaders
        (ShaderStage::Vertex, UI_VERT), // ui vert shader
        (ShaderStage::Fragment, UI_FRAG); // ui frag shader
        // provide the uniforms
        (RenderableComponent, UiMaterial, ShaderStage::Fragment), // ui draws on ui material
        (ObjectUniform, UiPosUniformObject, ShaderStage::Vertex), // ui uniforms
    )
}