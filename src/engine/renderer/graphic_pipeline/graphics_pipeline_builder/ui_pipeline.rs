use crate::{GraphicsPipelineBuilder, engine::{renderer::{shaders::{UI_VERT, UI_FRAG}, graphic_pipeline::uniform::{resource_uniform::textures_uniform::TextureUniformBuilder, uniform_buffer::UniformBufferBuilder, frame_uniform::ui_resolution::UiResolution}}, material::ui_material::UiMaterial}, ModelMatrixUniformObject};




impl GraphicsPipelineBuilder {
    pub fn ui_pipeline() -> GraphicsPipelineBuilder {
        GraphicsPipelineBuilder { 
            vertex_shader: (UI_VERT.iter().map(|v| *v).collect()),
            fragment_shader: (UI_FRAG.iter().map(|v| *v).collect()),
            resource_uniforms: vec![
                Box::new(TextureUniformBuilder::new(0, vulkanalia::vk::ShaderStageFlags::FRAGMENT))
            ],
            frame_uniforms: vec![
                Box::new(UniformBufferBuilder::<UiResolution>::new(vulkanalia::vk::ShaderStageFlags::VERTEX, vulkanalia::vk::DescriptorType::UNIFORM_BUFFER, 0)),
            ],
            object_uniforms: vec![
                Box::new(UniformBufferBuilder::<ModelMatrixUniformObject>::new(vulkanalia::vk::ShaderStageFlags::VERTEX, vulkanalia::vk::DescriptorType::STORAGE_BUFFER, 0)),
                Box::new(UniformBufferBuilder::<UiMaterial>::new(vulkanalia::vk::ShaderStageFlags::FRAGMENT, vulkanalia::vk::DescriptorType::STORAGE_BUFFER, 0)),
            ],
        }
    }
}