use crate::engine::material::Material;

#[cfg(feature = "ui")]
pub(crate) mod ui_mesh_renderer;

/// Component to render a Mesh.
pub struct MeshRenderer {
    mesh_id: u64,
    material: Material,
    instance: usize,
    is_static: bool,
}

impl MeshRenderer {
    pub fn new(
        mesh_id: u64,
        material: Material,
    ) -> MeshRenderer {
        MeshRenderer {
            mesh_id,
            material,
            instance: 0,
            is_static: false,
        }
    }

    pub fn new_static(
        mesh_id: u64,
        material: Material,
    ) -> MeshRenderer {
        MeshRenderer {
            mesh_id,
            material,
            instance: 0,
            is_static: true,
        }
    }

    pub fn pipeline_id(&self) -> u64 {
        self.material.pipeline_id()
    }

    pub fn instance(&self) -> usize {
        self.instance
    }

    pub fn set_instance(&mut self, instance: usize) {
        self.instance = instance;
    }

    pub fn material(&self) -> &Material {
        &self.material
    }

    pub fn mesh_id(&self) -> u64 {
        self.mesh_id
    }

    pub fn is_static(&self) -> bool {
        self.is_static
    }


}