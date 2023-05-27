use crate::engine::errors::PropellantError;

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::vk::DeviceV1_0;

/// Represent a vulkan type buffer.
pub struct VulkanBuffer {
    buffer: vulkanalia::vk::Buffer,
    memory: vulkanalia::vk::DeviceMemory,
    buffer_size: u64,
}

impl VulkanBuffer {
    pub fn create(
        instance: &vulkanalia::Instance,
        device: &vulkanalia::Device,
        physical_device: vulkanalia::vk::PhysicalDevice,
        buffer_size: vulkanalia::vk::DeviceSize,
        usage: vulkanalia::vk::BufferUsageFlags,
        properties: vulkanalia::vk::MemoryPropertyFlags,
    ) -> Result<VulkanBuffer, PropellantError> {
        let buffer_info = vulkanalia::vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(usage)
            .sharing_mode(vulkanalia::vk::SharingMode::EXCLUSIVE);
    
        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };
    
        let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
    
        let memory_info = vulkanalia::vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(Self::get_memory_type_index(
                instance,
                physical_device,
                properties,
                requirements,
            )?);
    
        let memory = unsafe { device.allocate_memory(&memory_info, None)? };
    
        unsafe { device.bind_buffer_memory(buffer, memory, 0)?; }
    
        Ok(VulkanBuffer {
            buffer,
            memory,
            buffer_size,
        })
    }

    fn get_memory_type_index(
        vk_instance: &vulkanalia::Instance,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        properties: vulkanalia::vk::MemoryPropertyFlags,
        requirements: vulkanalia::vk::MemoryRequirements
    ) -> Result<u32, PropellantError> {
        let memory = unsafe {vk_instance.get_physical_device_memory_properties(vk_physical_device) };
        (0..memory.memory_type_count)
            .find(|i| {
                let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
                let memory_type = memory.memory_types[*i as usize];
                suitable && memory_type.property_flags.contains(properties)
            })
            .ok_or(PropellantError::OutOfMemory)
    }

    pub fn map_data<T>(
        &mut self,
        vk_device: &vulkanalia::Device,
        data: &[T],
        offset: usize,
    ) -> Result<(), PropellantError> {
        // in debug mode, assert the mapped memory data will not overflow the buffer.
        debug_assert!(data.len() as u64 * std::mem::size_of::<T>() as u64 + offset as u64 <= self.buffer_size);

        let memory = unsafe { vk_device.map_memory(
            self.memory,
            offset as u64,
            data.len() as u64,
            vulkanalia::vk::MemoryMapFlags::empty(),
        )? };
    
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), memory.cast(), data.len());
        }
        unsafe {
            vk_device.unmap_memory(self.memory);
        }

        Ok(())
    }

    pub fn buffer(&self) -> vulkanalia::vk::Buffer {
        self.buffer
    }

    pub fn size(&self) -> u64 {
        self.buffer_size
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        unsafe {
            vk_device.destroy_buffer(self.buffer, None);
            vk_device.free_memory(self.memory, None);
        }
    }
}
