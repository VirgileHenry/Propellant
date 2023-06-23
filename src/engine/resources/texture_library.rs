use std::collections::{HashMap, BTreeMap};

use image::{ImageBuffer, Rgba};

use crate::{
    engine::{
        errors::PResult,
        window::vulkan::{
            transfer_command_manager::TransferCommandManager,
            vulkan_buffer::VulkanBuffer,
            vulkan_image::VulkanImage
        }
    },
    id
};

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;

/// A texture allocated on the gpu for easy access.
pub struct LoadedTexture {
    /// The texture buffer on the gpu.
    texture: VulkanImage,
    /// the image view to access the texture.
    view: vulkanalia::vk::ImageView,
    /// The image sampler.
    sampler: vulkanalia::vk::Sampler,
}

impl LoadedTexture {
    pub fn create(
        from: ImageBuffer<Rgba<u8>, Vec<u8>>,
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
        // create the image object
        let texture = VulkanImage::create(
            vk_instance,
            vk_device,
            vk_physical_device,
            from.width(),
            from.height(),
            vulkanalia::vk::ImageUsageFlags::SAMPLED | vulkanalia::vk::ImageUsageFlags::TRANSFER_DST,
            vulkanalia::vk::Format::R8G8B8A8_SRGB,
        )?;

        // ask the transfer manager to send the buffer to the image object
        vk_transfer_manager.register_image_transfer(
            vk_device,
            staging_buffer,
            texture.image(),
            from.width(),
            from.height(),
        )?;

        // create the image view. 
        // we do create it before the transfer is done, should be fine.
        let subresource_range = vulkanalia::vk::ImageSubresourceRange::builder()
            .aspect_mask(vulkanalia::vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let info = vulkanalia::vk::ImageViewCreateInfo::builder()
            .image(texture.image())
            .view_type(vulkanalia::vk::ImageViewType::_2D)
            .format(vulkanalia::vk::Format::R8G8B8A8_SRGB)
            .subresource_range(subresource_range);

        let view = unsafe { vk_device.create_image_view(&info, None)? };

        // create the sampler
        // todo : all of these should be configurable at texture creation ?
        // todo : anisotropy sampling should be disabled if not supported
        let info = vulkanalia::vk::SamplerCreateInfo::builder()
            .mag_filter(vulkanalia::vk::Filter::LINEAR)
            .min_filter(vulkanalia::vk::Filter::LINEAR)
            .address_mode_u(vulkanalia::vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vulkanalia::vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vulkanalia::vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(true)
            .max_anisotropy(16.0)
            .border_color(vulkanalia::vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vulkanalia::vk::CompareOp::ALWAYS)
            .mipmap_mode(vulkanalia::vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0);
        
        let sampler = unsafe { vk_device.create_sampler(&info, None)? };

        // return the created image object
        Ok(LoadedTexture {
            texture,
            view,
            sampler,
        })
    }

    pub fn view(&self) -> vulkanalia::vk::ImageView {
        self.view
    }

    pub fn sampler(&self) -> vulkanalia::vk::Sampler {
        self.sampler
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device
    ) {
        unsafe {
            vk_device.destroy_image_view(self.view, None);
            vk_device.destroy_sampler(self.sampler, None);
        }
        self.texture.destroy(vk_device);
    }
}




pub struct TextureLibrary {
    /// The texture hash id, mapped to the texture raw bytes and it's index in the texture buffer.
    loading_queue: HashMap<u64, (ImageBuffer<Rgba<u8>, Vec<u8>>, u32)>,
    /// The texture index, mapped to the texture id and object.
    textures: BTreeMap<u32, (u64, LoadedTexture)>,
    next_texture_index: u32,
}

impl TextureLibrary {
    pub fn new() -> TextureLibrary {
        
        let mut loading_queue = HashMap::new();
        loading_queue.insert(id("white"), (Self::create_white_textures(), 0));

        TextureLibrary {
            loading_queue,
            textures: BTreeMap::new(),
            next_texture_index: 1, // 0 is for the white texture
        }
    }

    /// Register a texture to be queued for loading.
    /// The texture will be loaded in memory the next time the renderer update,
    /// and the flag `RequireResourcesLoadingFlag` is set to textures.
    /// This operation might fail if the bytes are not a valid image.
    /// This will return the texture index, so it can then be used by a material to reference it.
    pub fn register_texture(&mut self, texture_id: u64, bytes: &[u8]) -> PResult<u32> {
        let index = self.next_texture_index;
        self.next_texture_index += 1;
        self.loading_queue.insert(texture_id, (image::load_from_memory(bytes)?.to_rgba8(), index));
        Ok(index)
    }

    pub fn load_textures(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_transfer_manager: &mut TransferCommandManager,
    ) -> PResult<()> {
        for (id, (bytes, index)) in self.loading_queue.drain() {
            let loaded_texture = LoadedTexture::create(
                bytes,
                vk_instance,
                vk_device,
                vk_physical_device,
                vk_transfer_manager,
            )?;
            self.textures.insert(index, (id, loaded_texture));
        }
        Ok(())
    }

    fn create_white_textures() -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 255, 255, 255]))
    }

    pub fn textures(&self) -> impl Iterator<Item = (u32, &LoadedTexture)> {
        self.textures.iter().map(|(index, (_, texture))| (*index, texture))
    }

    pub fn max_index(&self) -> u32 {
        self.next_texture_index - 1
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device
    ) {
        for (_, (_, texture)) in self.textures.iter_mut() {
            texture.destroy(vk_device);
        }
        self.textures.clear();
    }
}