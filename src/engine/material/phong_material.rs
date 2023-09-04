use foundry::AsAny;

use crate::{
    engine::renderer::graphic_pipeline::renderable_component::RenderableComponent,
    InstancedMeshRenderer
};

use super::colored_texture::ColoredTexture;

#[repr(C)]
#[allow(unused)]
#[derive(Debug, Clone, AsAny)]
pub struct PhongMaterial {
    albedo: ColoredTexture, // default color
    metalic: ColoredTexture, // sininess color ?
}

impl PhongMaterial {
    pub fn colored(mut self, color: glam::Vec3) -> Self {
        self.albedo.set_color(color);
        self
    }

    pub fn textured(mut self, texture_index: u32) -> Self {
        self.albedo.set_texture(texture_index);
        self
    }
}

impl RenderableComponent for PhongMaterial {
    type FromComponent<Mesh> = InstancedMeshRenderer<PhongMaterial, Mesh>;

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

impl Default for PhongMaterial {
    fn default() -> Self {
        PhongMaterial {
            albedo: ColoredTexture::color(glam::Vec3::ONE),
            metalic: ColoredTexture::color(glam::Vec3::ZERO),
        }
    }
}
