#[cfg(feature = "ui")]
pub(crate) mod ui_mesh_renderer;

/// Component to render a Mesh.
pub struct InstancedMeshRenderer<Material> {
    mesh_id: u64,
    material: Material,
    uniform_buffer_offset: usize,
}

impl<Material> InstancedMeshRenderer<Material> {
    pub fn new(
        mesh_id: u64,
        material: Material,
    ) -> InstancedMeshRenderer<Material> {
        InstancedMeshRenderer {
            mesh_id,
            material,
            uniform_buffer_offset: 0,
        }
    }

    #[inline]
    pub fn set_uniform_buffer_offset(&mut self, offset: usize) {
        self.uniform_buffer_offset = offset;
    }

    #[inline]
    pub fn uniform_buffer_offset(&self) -> usize {
        self.uniform_buffer_offset
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn mesh_id(&self) -> u64 {
        self.mesh_id
    }
}
