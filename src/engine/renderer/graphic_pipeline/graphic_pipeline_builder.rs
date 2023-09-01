use std::collections::HashMap;

use crate::engine::errors::PResult;

use super::GraphicPipelineInterface;

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


pub fn default_phong_pipeline() -> impl GraphicPipelineBuilderInterface {
    use crate::CameraUniformObject;
    use crate::MainDirectionnalLight;
    use crate::ModelMatrixUniformObject;
    use crate::PhongMaterial;
    use crate::create_graphic_pipeline;
    use crate::engine::renderer::shaders::DEFAULT_FRAG;
    use crate::engine::renderer::shaders::DEFAULT_VERT;
    use super::renderable_component::RenderableComponent;
    use super::uniform::frame_uniform::FrameUniform;
    use super::uniform::object_uniform::ObjectUniform;
    use crate::ShaderStage;
    use crate::engine::renderer::graphic_pipeline::GraphicPipelineCreationState;
    create_graphic_pipeline!(
        (ShaderStage::Vertex, DEFAULT_VERT), // vert shader
        (ShaderStage::Fragment, DEFAULT_FRAG); // frag shader
        (FrameUniform, CameraUniformObject, ShaderStage::Vertex), // camera uniforms (one per frame)
        (FrameUniform, MainDirectionnalLight, ShaderStage::Fragment), // light uniforms (one per frame)
        (RenderableComponent, PhongMaterial, ShaderStage::Fragment), // phong material (one per object)
        (ObjectUniform, ModelMatrixUniformObject, ShaderStage::Vertex), // model matrix (one per object)
        (TexturesUniform, ShaderStage::Fragment), // textures
    )
}

#[cfg(feature = "ui")]
pub fn default_ui_pipeline() -> impl GraphicPipelineBuilderInterface {
use crate::create_graphic_pipeline;
    use super::renderable_component::RenderableComponent;
    use crate::engine::renderer::shaders::UI_FRAG;
    use crate::engine::renderer::shaders::UI_VERT;
    use super::uniform::object_uniform::ObjectUniform;
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


