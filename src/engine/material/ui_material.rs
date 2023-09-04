use foundry::AsAny;

use crate::{
    ColoredTexture,
    engine::renderer::graphic_pipeline::renderable_component::RenderableComponent,
    InstancedMeshRenderer,
    id, StaticMesh
};


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

    pub fn to_mesh_renderer(self) -> InstancedMeshRenderer<UiMaterial, StaticMesh> {
        InstancedMeshRenderer::<UiMaterial, StaticMesh>::new(id("ui_quad"), self)
    }
}

impl RenderableComponent for UiMaterial {
    type FromComponent<Mesh> = InstancedMeshRenderer<UiMaterial, Mesh>;

    fn set_uniform<Mesh>(component: &Self::FromComponent<Mesh>, write_to_buf: &mut dyn FnMut(&[Self], usize), instance_count: usize) {
        for i in 0..instance_count {
            write_to_buf(&[component.material().clone()], i);
        }
    }

    fn mesh_id<Mesh>(component: &Self::FromComponent<Mesh>) -> u64 {
        component.mesh_id()
    }

    fn set_uniform_buffer_index<Mesh>(component: &mut Self::FromComponent<Mesh>, index: usize) {
        component.set_uniform_buffer_offset(index);
    }

    fn uniform_buffer_index<Mesh>(component: &Self::FromComponent<Mesh>) -> usize {
        component.uniform_buffer_offset()
    }

    fn instance_count<Mesh>(_component: &Self::FromComponent<Mesh>) -> usize {
        1
    }
}