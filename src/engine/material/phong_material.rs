use foundry::AsAny;

use crate::{engine::renderer::graphic_pipeline::{uniform::object_uniform::ObjectUniform, renderable_component::RenderableComponent}, InstancedMeshRenderer};

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

impl ObjectUniform for PhongMaterial {
    type FromComponent = InstancedMeshRenderer<PhongMaterial>;
    fn get_uniform(component: &Self::FromComponent) -> Self {
        component.material().clone()
    }
}

impl RenderableComponent for PhongMaterial {
    fn mesh_id(component: &Self::FromComponent) -> u64 {
        component.mesh_id()
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
