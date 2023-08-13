use foundry::AsAny;

use crate::{ColoredTexture, engine::renderer::graphic_pipeline::{renderable_component::RenderableComponent, uniform::object_uniform::ObjectUniform}, InstancedMeshRenderer, id};


#[allow(unused)]
#[repr(C)]
#[derive(Debug, Clone, Copy, AsAny)]
pub struct UiMaterial {
    pub texture: ColoredTexture,
    pub corner_radius: f32,
    /// experimentally correct
    _padding: [f32; 3],
}

impl UiMaterial {
    pub fn new(texture: ColoredTexture, corner_radius: f32) -> UiMaterial {
        UiMaterial {
            texture,
            corner_radius,
            _padding: [0.0; 3],
        }
    }

    pub fn colored(color: glam::Vec3, corner_radius: f32) -> UiMaterial {
        UiMaterial {
            texture: ColoredTexture::color(color),
            corner_radius,
            _padding: [0.0; 3],
        }
    }

    pub fn to_mesh_renderer(self) -> InstancedMeshRenderer<UiMaterial> {
        InstancedMeshRenderer::new(id("ui_quad"), self)
    }
}

impl ObjectUniform for UiMaterial {
    type FromComponent = InstancedMeshRenderer<UiMaterial>;
    fn get_uniform(component: &Self::FromComponent) -> Self {
        component.material().clone()
    }
}

impl RenderableComponent for UiMaterial {
    fn mesh_id(component: &Self::FromComponent) -> u64 {
        component.mesh_id()
    }

    fn set_uniform_buffer_index(component: &mut Self::FromComponent, index: usize) {
        component.set_uniform_buffer_offset(index);
    }

    fn uniform_buffer_index(component: &Self::FromComponent) -> usize {
        component.uniform_buffer_offset()
    }
}