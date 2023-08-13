use crate::resource_loading::RequireResourcesLoadingFlag;

use self::{mesh_library::MeshLibrary, texture_library::TextureLibrary};
use super::{
    window::vulkan::transfer_command_manager::TransferCommandManager,
    errors::PResult
};

pub(crate) mod mesh_library;
pub(crate) mod texture_library;

/// Holds all the resources that are required by the user, 3D models, textures, etc.
pub struct PropellantResources {
    meshes: MeshLibrary,
    textures: TextureLibrary,
}

#[cfg(feature = "ui")]
impl Default for PropellantResources {
    fn default() -> Self {
        PropellantResources {
            meshes: MeshLibrary::with_ui_quad(),
            textures: TextureLibrary::new(),
        }
    }
}

#[cfg(not(feature = "ui"))]
impl Default for PropellantResources {
    fn default() -> Self {
        PropellantResources {
            meshes: MeshLibrary::new(),
            textures: TextureLibrary::new(),
        }
    }
}

impl PropellantResources {

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

        if flags.contains(RequireResourcesLoadingFlag::TEXTURES) {
            self.textures.load_textures(vk_instance, vk_device, vk_physical_device, vk_transfer_manager)?;
        }

        Ok(())
    }

    pub fn meshes(&self) -> &MeshLibrary {
        &self.meshes
    }

    pub fn meshes_mut(&mut self) -> &mut MeshLibrary {
        &mut self.meshes
    }

    pub fn textures(&self) -> &TextureLibrary {
        &self.textures
    }

    pub fn textures_mut(&mut self) -> &mut TextureLibrary {
        &mut self.textures
    }

    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.meshes.destroy(vk_device);
        self.textures.destroy(vk_device);
    }
}