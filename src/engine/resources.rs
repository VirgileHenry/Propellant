use crate::MeshLibrary;

use super::{window::vulkan::transfer_command_manager::TransferCommandManager, flags::RequireResourcesLoadingFlag, errors::PResult};



/// Holds all the resources that are required by the user, 3D models, textures, etc.
pub struct ProppellantResources {
    meshes: MeshLibrary,
}

impl Default for ProppellantResources {
    fn default() -> Self {
        ProppellantResources {
            meshes: MeshLibrary::new(),
        }
    }
}

impl ProppellantResources {

    pub fn load_resources(
        &mut self,
        flags: RequireResourcesLoadingFlag,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<()> {
        // check for mesh loading
        if flags.contains(RequireResourcesLoadingFlag::MESHES) {
            self.meshes.load_meshes(vk_instance, vk_device, vk_physical_device, vk_transfer_manager)?;
        }

        Ok(())
    }

    pub fn meshes(&self) -> &MeshLibrary {
        &self.meshes
    }

    pub fn meshes_mut(&mut self) -> &mut MeshLibrary {
        &mut self.meshes
    }

    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.meshes.destroy(vk_device);
    }
}