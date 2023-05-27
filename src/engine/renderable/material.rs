use crate::id;



/// Info on how to draw a mesh renderer.
pub struct Material {
    /// The graphic pipeline used to draw the mesh.
    graphic_pipeline: u64,
}

impl Material {
    pub fn pipeline_id(&self) -> u64 {
        self.graphic_pipeline
    }
}

impl Default for Material {
    fn default() -> Self {
        Material {
            graphic_pipeline: id("default"),
        }
    }
}