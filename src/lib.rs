pub(crate) mod engine;
pub(crate) mod utils;

// expose our types
pub use engine::{
    PropellantEngine,
    common_components::{
        camera::Camera,
    },
    common_systems::fps_limiter::FpsLimiter,
    inputs::{
        input_context::InputContext,
        input_handler::input_handler_builder::InputHandlerBuilder,
        input_handler::InputHandler,
    },
    mesh::{
        Mesh,
        mesh_renderer::MeshRenderer,
    },
    material::{
        Material,
        phong_material::PhongMaterialProperties,
    },
    engine_events::PropellantEvent,
    resources::ProppellantResources,
    window::{
        PropellantWindow,
        window_builder::PropellantWindowBuilder,
        vulkan::vulkan_interface::VulkanInterface,
    },
    transform::Transform,
    lights::{
        directionnal_light::DirectionnalLight,
    },
    flags::*,
    renderer::{
        renderer_builder::default_vulkan_renderer_builder::DefaultVulkanRendererBuilder,
        rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder,
        rendering_pipeline::rendering_pipeline_builder::rendering_pipeline_layer::RenderingPipelineLayer,
        graphics_pipeline::graphics_pipeline_builder::GraphicsPipelineBuilder,
        rendering_pipeline::final_render_target::FinalRenderTargetBuilder,
        rendering_pipeline::intermediate_render_targets::IntermediateRenderTargetBuilder,
    },
};

pub use foundry;
pub use glam;

pub use utils::id::{
    id,
    small_id,
};


