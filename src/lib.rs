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
        vulkan::vulkan_interface::VulkanInterface,
    },
    transform::Transform,
    lights::{
        directionnal_light::DirectionnalLight,
    },
};

pub use utils::id::{
    id,
    small_id,
};


