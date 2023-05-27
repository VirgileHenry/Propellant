#[macro_use] extern crate glsl_to_spirv_macros;
#[macro_use] extern crate glsl_to_spirv_macros_impl;

pub(crate) mod engine;
pub(crate) mod utils;

// expose our types
pub use engine::{
    PropellantEngine,
    engine_events::{
        input_handler::InputHandler,
        input_listener::{
            InputListener,
            input_button::InputButton,
        },
    },
    renderable::{
        mesh::{
            Mesh,
            mesh_renderer::MeshRenderer,
            mesh_renderer_builder::MeshRendererBuilder,
        },
        material::Material,
    },
    window::{
        PropellantWindow,
        vulkan::vulkan_interface::VulkanInterface,
    },
    transform::Transform,
};

pub use utils::id::id;


