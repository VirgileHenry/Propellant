use std::collections::HashMap;

use crate::{Mesh, Material, engine::window::vulkan::transfer_command_manager::TransferCommandManager};

use super::instance_mesh_lib::{InstanceMeshLib, InstanceMesh};




pub struct InstanceMeshLibBuilder {
    meshes: HashMap<u64, (Mesh, Material)>,
}

impl InstanceMeshLibBuilder {
    pub fn new() -> InstanceMeshLibBuilder {
        InstanceMeshLibBuilder {
            meshes: HashMap::new(),
        }
    }

    pub fn register_mesh(&mut self, id: u64, mesh: Mesh, material: Material) {
        self.meshes.insert(id, (mesh, material));
    }

    pub fn build(
        self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> InstanceMeshLib {
        let meshes = self.meshes.into_iter()
            .map(
                |(id, (mesh, material))| (id, InstanceMesh::build(
                    mesh,
                    material,
                    vk_instance,
                    vk_device,
                    vk_physical_device,
                    vk_transfer_manager
                )
            ))
            .filter_map(
                |(id, result)| match result {
                    Ok(mesh) => Some((id, mesh)),
                    Err(e) => {
                        println!("[PROPELLANT] Error while building mesh for instance rendering : {e:?}");
                        None
                    }
                }
            )
            .collect::<HashMap<_, _>>();

        InstanceMeshLib::new(meshes)
    }
}

