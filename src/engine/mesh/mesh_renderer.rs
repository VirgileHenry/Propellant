#[cfg(feature = "ui")]
pub(crate) mod ui_mesh_renderer;

/// Component to render a Mesh.
pub struct InstancedMeshRenderer<Material, Mesh> {
    mesh_type: std::marker::PhantomData<Mesh>,
    mesh_id: u64,
    material: Material,
    uniform_buffer_offset: usize,
}

impl<Material, Mesh> InstancedMeshRenderer<Material, Mesh> {
    pub fn new(
        mesh_id: u64,
        material: Material,
    ) -> InstancedMeshRenderer<Material, Mesh> {
        InstancedMeshRenderer {
            mesh_type: std::marker::PhantomData,
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
