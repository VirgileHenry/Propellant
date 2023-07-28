pub(crate) mod engine;
pub(crate) mod utils;

// expose our types
pub use engine::{
    PropellantEngine,
    common_components::camera::Camera,
    common_systems::fps_limiter::FpsLimiter,
    inputs::{
        input_context::InputContext,
        input_handler::input_handler_builder::InputHandlerBuilder,
        input_handler::InputHandler,
    },
    mesh::{
        Mesh,
        mesh_renderer::MeshRenderer,
        Vertex,
    },
    material::{
        Material,
        phong_material::PhongMaterialProperties,
        colored_texture::ColoredTexture,
    },
    engine_events::PropellantEvent,
    resources::ProppellantResources,
    window::{
        PropellantWindow,
        window_builder::PropellantWindowBuilder,
        vulkan::vulkan_interface::VulkanInterface,
    },
    transform::{
        ui_transform::UiTransform,
        transform::Transform,
    },
    lights::directionnal_light::DirectionnalLight,
    flags::*,
    renderer::{
        renderer_builder::default_vulkan_renderer_builder::DefaultVulkanRendererBuilder,
        rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder,
        rendering_pipeline::rendering_pipeline_builder::rendering_pipeline_layer::RenderingPipelineLayer,
        graphics_pipeline::{
            graphics_pipeline_builder::GraphicsPipelineBuilder,
            uniform::frame_uniform::AsPerFrameUniform,
            uniform::object_uniform::AsPerObjectUniform,
            uniform::frame_uniform::{
                camera_uniform::CameraUniformObject,
                main_directionnal_light::MainDirectionnalLight,
            },
            uniform::object_uniform::model_uniform::ModelMatrixUniformObject,
        },
        rendering_pipeline::final_render_target::FinalRenderTargetBuilder,
        rendering_pipeline::intermediate_render_targets::IntermediateRenderTargetBuilder,
    },

};

#[cfg(feature = "ui")]
pub use engine::{
    material::ui_material::UiMaterial,
    transform::ui_transform::UiAnchor,
    ui::ui_event_listener::{
        UiEventListener,
        UiListenerCallback,
    },
    inputs::common_context::ui_event_context::{
        ui_events::UiEvent,
        CursorPosition,
    },
};

pub use foundry;
pub use glam;
pub use winit;
pub use vulkanalia;
pub use vk_shader_macros;

pub use utils::id::{
    id,
    small_id,
};


