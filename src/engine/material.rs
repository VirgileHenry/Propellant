use crate::id;
pub(crate) mod default_material;


/// Info on how to draw a mesh renderer.
pub struct Material {
    /// The graphic pipeline used to draw the mesh.
    graphic_pipeline: u64,
    // material properties
    // todo : maybe a super trait ?
    // properties: Box<dyn ToBuffer>,
}

impl Material {
    pub fn pipeline_id(&self) -> u64 {
        self.graphic_pipeline
    }
}

impl Default for Material {
    fn default() -> Self {
        Material {
            graphic_pipeline: id("default"), // default pipeline rendering.
            // properties: Box::new(DefaultMaterial::default()),
        }
    }
}