use foundry::AsAny;

use crate::id;

use self::phong_material::PhongMaterialProperties;

use super::{
    renderer::graphics_pipeline::uniform::object_uniform::AsPerObjectUniform,
    errors::{
        PResult, PropellantError,
        rendering_error::RenderingError
    }
};

pub(crate) mod phong_material;
pub(crate) mod colored_texture;

/// Info on how to draw a mesh renderer.
pub struct Material {
    /// The graphic pipeline used to draw the mesh.
    graphic_pipeline: u64,
    // material properties
    properties: Box<dyn AsAny>,
}


impl Material {
    pub fn pipeline_id(&self) -> u64 {
        self.graphic_pipeline
    }

    pub fn properties(&self) -> &dyn AsAny {
        self.properties.as_ref()
    }

    pub fn with_prop<T: AsAny + 'static>(mut self, properties: T) -> Material {
        self.properties = Box::new(properties);
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Material {
            graphic_pipeline: id("default"), // default pipeline rendering.
            properties: Box::new(PhongMaterialProperties::default()),
        }
    }
}

impl<T: Clone + AsAny + 'static> AsPerObjectUniform for T {
    fn get_uniform(_transform: &crate::Transform, material: &crate::Material) -> PResult<Self> where Self: Sized {
        // really bad, this kind of cast per uniform update.
        // it'll do it for now, + materials should be updated every frame.
        material.properties().as_any().downcast_ref::<T>()
            .and_then(|p| Some(p.clone()))
            .ok_or(PropellantError::Rendering(RenderingError::InvalidMaterialCast))
    }
}