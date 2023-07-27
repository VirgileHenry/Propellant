use crate::{MeshRenderer, id, engine::material::ui_material::UiMaterial, Material};




impl MeshRenderer {
    pub fn ui_renderer(material: UiMaterial) -> MeshRenderer {
        MeshRenderer::new(
            id("ui_quad"),
            Material::new(id("ui_pipeline"), material)
        )
    }
}