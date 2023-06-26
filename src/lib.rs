pub(crate) mod engine;
pub(crate) mod utils;

// expose our types
pub use engine::{
    common_components::camera::Camera,
    PropellantEngine,
    engine_events::{
        input_handler::InputHandler,
        input_listener::{
            InputListener,
            input_button::InputButton,
        },
    },
    mesh::{
        Mesh,
        mesh_renderer::MeshRenderer,
    },
    material::{
        Material,
        phong_material::PhongMaterialProperties,
    },
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
    },
};

pub use utils::id::{
    id,
    small_id,
};


