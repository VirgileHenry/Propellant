pub(crate) mod engine;
pub(crate) mod utils;

// expose our types
pub use engine::{
    PropellantEngine,
    common_components::camera::Camera,
    common_systems::fps_limiter::FpsLimiter,
    mesh::{
        Mesh,
        mesh_renderer::InstancedMeshRenderer,
        Vertex,
    },
    material::{
        phong_material::PhongMaterial,
        colored_texture::ColoredTexture,
    },
    engine_events::{
        PropellantEvent,
        PropellantEventSenderExt,
    },
    resources::PropellantResources,
    window::{
        PropellantWindow,
        window_builder::PropellantWindowBuilder,
        vulkan::vulkan_interface::VulkanInterface,
    },
    transform::transform::Transform,
    lights::directionnal_light::DirectionnalLight,
    flags::*,
    renderer::{
        renderer_builder::default_vulkan_renderer_builder::DefaultVulkanRendererBuilder,
        rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder,
        graphic_pipeline::{
            uniform::frame_uniform::{
                camera_uniform::CameraUniformObject,
                main_directionnal_light::MainDirectionnalLight,
            },
            uniform::object_uniform::model_uniform::ModelMatrixUniformObject,
            graphic_pipeline_builder::default_phong_pipeline,
            graphic_pipeline_gen::ShaderStage,
        },
        rendering_pipeline::final_render_target::FinalRenderTargetBuilder,
        rendering_pipeline::intermediate_render_targets::IntermediateRenderTargetBuilder,
    },

};

#[cfg(feature = "ui")]
pub use engine::{
    material::ui_material::UiMaterial,
    transform::ui_transform::{
        UiAnchor,
        UiTransform,
    },
};

#[cfg(feature = "inputs")]
pub use engine::inputs::{
    input_context::InputContext,
    input_handler::input_handler_builder::InputHandlerBuilder,
    input_handler::InputHandler,
};

#[cfg(all(feature = "ui", feature = "inputs"))]
pub use engine::{
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

pub use utils::{
    id::{
        id,
        small_id,
    },
    builder::HasBuilder,
};


