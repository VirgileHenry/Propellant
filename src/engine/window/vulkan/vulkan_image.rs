use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;

use crate::engine::errors::PResult;


pub struct VulkanImage {
    image: vulkanalia::vk::Image,
    memory: vulkanalia::vk::DeviceMemory,
    image_size: u64,
    image_format: vulkanalia::vk::Format,
    image_layout: vulkanalia::vk::ImageLayout,
}

impl VulkanImage {
    pub fn create(
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        usage: vulkanalia::vk::ImageUsageFlags,
        width: u32,
        height: u32,
    ) -> PResult<VulkanImage> {
       // image create info
       let info = vulkanalia::vk::ImageCreateInfo::builder()
            .image_type(vulkanalia::vk::ImageType::_2D)
            .extent(vulkanalia::vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .format(vulkanalia::vk::Format::R8G8B8A8_SRGB)
            .tiling(vulkanalia::vk::ImageTiling::OPTIMAL)
            .initial_layout(vulkanalia::vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vulkanalia::vk::SharingMode::EXCLUSIVE)
            .samples(vulkanalia::vk::SampleCountFlags::_1);

        let image = unsafe { vk_device.create_image(&info, None)? };

        /*
        let requirements = unsafe { vk_device.get_image_memory_requirements(image) };
        
        let info = vulkanalia::vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(
            vk_instance,
            properties,
            requirements,
        )?);
        
        let image_memory = unsafe { vk_device.allocate_memory(&info, None)? };
        
        unsafe {
            vk_device.bind_image_memory(image, image_memory, 0)?;
        }
        */
       
       unimplemented!()
    }
}