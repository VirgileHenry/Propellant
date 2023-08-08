#[cfg(feature = "ui")]
pub(crate) mod ui_mesh_renderer;

/// Component to render a Mesh.
pub struct InstancedMeshRenderer<Material> {
    mesh_id: u64,
    material: Material,
}

impl<Material> InstancedMeshRenderer<Material> {
    pub fn new(
        mesh_id: u64,
        material: Material,
    ) -> InstancedMeshRenderer<Material> {
        InstancedMeshRenderer {
            mesh_id,
            material,
        }
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn mesh_id(&self) -> u64 {
        self.mesh_id
    }
}
