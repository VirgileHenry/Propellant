use std::collections::HashMap;

use image::{ImageBuffer, Rgba};

use crate::{engine::{errors::PResult, window::vulkan::{transfer_command_manager::TransferCommandManager, vulkan_buffer::VulkanBuffer}}, id};



/// A texture allocated on the gpu for easy access.
pub struct LoadedTexture {
    /// The index of the texture in the texture buffer on the gpu.
    loaded_texture_index: u32,
}

impl LoadedTexture {
    pub fn create(
        from: ImageBuffer<Rgba<u8>, Vec<u8>>,
        id: u32,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<LoadedTexture> {
        let image_size = from.len() as u64;
        // create a staging buffer for our image
        let mut staging_buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            image_size,
            vulkanalia::vk::BufferUsageFlags::TRANSFER_SRC,
            vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT | vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;
        // map the image data to the staging buffer
        staging_buffer.map_data(
            vk_device,
            &from.as_raw(),
            0,
        )?;
        // create an image object

        // ask the transfer manager to send the buffer to the image object

        // return the created image object

        unimplemented!()
    }
}




pub struct TextureLibrary {
    loading_queue: HashMap<u64, ImageBuffer<Rgba<u8>, Vec<u8>>>,
    textures: HashMap<u64, LoadedTexture>,
    next_texture_index: u32,
}

impl TextureLibrary {
    pub fn new() -> TextureLibrary {
        
        let mut loading_queue = HashMap::new();
        loading_queue.insert(id("white"), Self::create_white_textures());

        TextureLibrary {
            loading_queue,
            textures: HashMap::new(),
            next_texture_index: 1, // 0 is for the white texture
        }
    }

    pub fn register_texture(&mut self, texture_id: u64, bytes: &[u8]) -> PResult<()> {
        self.loading_queue.insert(texture_id, image::load_from_memory(bytes)?.to_rgba8());
        Ok(())
    }

    pub fn load_textures(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<()> {
        for (id, bytes) in self.loading_queue.drain() {
            let loaded_texture = LoadedTexture::create(
                bytes,
                self.next_texture_index,
                vk_instance,
                vk_device,
                vk_physical_device,
                vk_transfer_manager,
            )?;
            self.textures.insert(id, loaded_texture);
            self.next_texture_index += 1;
        }
        Ok(())
    }

    fn create_white_textures() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]))
    }

    pub fn index_from_id(&self, id: u64) -> Option<u32> {
        self.textures.get(&id).map(|t| t.loaded_texture_index)
    }
}