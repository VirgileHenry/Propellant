use std::collections::HashMap;

use crate::engine::{errors::PResult, window::vulkan::{transfer_command_manager::TransferCommandManager, vulkan_buffer::VulkanBuffer}};



/// A texture allocated on the gpu for easy access.
pub struct LoadedTexture {

}

impl LoadedTexture {
    pub fn load_bytes(
        bytes: Vec<u8>,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<LoadedTexture> {
        let image = image::load_from_memory(&bytes)?.to_rgba8();
        let image_size = image.len() as u64;
        // create a staging buffer for our image
        let mut staging_buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            image_size,
            vulkanalia::vk::BufferUsageFlags::TRANSFER_SRC,
            vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT | vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        // mapping the image data to the staging buffer
        staging_buffer.map_data(
            vk_device,
            &image.as_raw(),
            0,
        )?;
        


        unimplemented!()
    }
}




pub struct TextureLibrary {
    loading_queue: HashMap<u64, Vec<u8>>,
    textures: HashMap<u64, LoadedTexture>,
}

impl TextureLibrary {
    pub fn new() -> TextureLibrary {
        TextureLibrary {
            loading_queue: HashMap::new(),
            textures: HashMap::new(),
        }
    }

    pub fn register_texture(&mut self, texture_id: u64, texture: &[u8]) {
        self.loading_queue.insert(texture_id, texture.to_vec());
    }

    pub fn load_textures(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<()> {
        for (id, bytes) in self.loading_queue.drain() {
            let loaded_texture = LoadedTexture::load_bytes(
                bytes,
                vk_instance,
                vk_device,
                vk_physical_device,
                vk_transfer_manager,
            )?;
            self.textures.insert(id, loaded_texture);
        }
        Ok(())
    }
}