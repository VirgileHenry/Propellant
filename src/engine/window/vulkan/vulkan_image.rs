use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::InstanceV1_0;

use crate::engine::errors::PResult;
use crate::engine::errors::PropellantError;

pub(crate) mod vulkan_image_view;

#[allow(unused)] // width, height and format are not used yet, maybe remove those fields ?
pub struct VulkanImage {
    image: vulkanalia::vk::Image,
    memory: vulkanalia::vk::DeviceMemory,
    width: u32,
    height: u32,
    image_format: vulkanalia::vk::Format,
}

impl VulkanImage {
    pub fn create(
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        width: u32,
        height: u32,
        usage: vulkanalia::vk::ImageUsageFlags,
        format: vulkanalia::vk::Format,
    ) -> PResult<VulkanImage> {
        // image create info
        let info = vulkanalia::vk::ImageCreateInfo::builder()
            .image_type(vulkanalia::vk::ImageType::_2D)
            .extent(vulkanalia::vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vulkanalia::vk::ImageTiling::OPTIMAL)
            .initial_layout(vulkanalia::vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .sharing_mode(vulkanalia::vk::SharingMode::EXCLUSIVE)
            .samples(vulkanalia::vk::SampleCountFlags::_1);

        let image = unsafe { vk_device.create_image(&info, None)? };

        let requirements = unsafe { vk_device.get_image_memory_requirements(image) };
        
        let info = vulkanalia::vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(Self::get_memory_type_index(
                vk_instance,
                vk_physical_device,
                vulkanalia::vk::MemoryPropertyFlags::DEVICE_LOCAL,
                requirements,
            )?);
        
        let memory = unsafe { vk_device.allocate_memory(&info, None)? };
        
        unsafe {
            vk_device.bind_image_memory(image, memory, 0)?;
        }

        Ok(VulkanImage {
            image,
            memory,
            width,
            height,
            image_format: format,
        })
    }

    fn get_memory_type_index(
        vk_instance: &vulkanalia::Instance,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        properties: vulkanalia::vk::MemoryPropertyFlags,
        requirements: vulkanalia::vk::MemoryRequirements
    ) -> PResult<u32> {
        let memory = unsafe {vk_instance.get_physical_device_memory_properties(vk_physical_device) };
        (0..memory.memory_type_count)
            .find(|i| {
                let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
                let memory_type = memory.memory_types[*i as usize];
                suitable && memory_type.property_flags.contains(properties)
            })
            .ok_or(PropellantError::OutOfMemory)
    }

    pub fn image(&self) -> vulkanalia::vk::Image {
        self.image
    }

    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        unsafe {
            vk_device.destroy_image(self.image, None);
            vk_device.free_memory(self.memory, None);
        }
    }
}