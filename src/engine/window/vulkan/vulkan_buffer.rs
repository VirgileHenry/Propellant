use crate::engine::consts::PROPELLANT_DEBUG_FEATURES;
use crate::engine::errors::PResult;
use crate::engine::errors::PropellantError;

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::Handle;

/// Represent a vulkan type buffer.
#[derive(Debug)]
pub struct VulkanBuffer {
    buffer: vulkanalia::vk::Buffer,
    memory: vulkanalia::vk::DeviceMemory,
    buffer_size: u64,
}

impl VulkanBuffer {

    /// Create a new empty buffer, without any memory allocated.
    pub fn empty() -> VulkanBuffer {
        VulkanBuffer {
            buffer: vulkanalia::vk::Buffer::null(),
            memory: vulkanalia::vk::DeviceMemory::null(),
            buffer_size: 0,
        }
    }

    pub fn create(
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        buffer_size: vulkanalia::vk::DeviceSize,
        usage: vulkanalia::vk::BufferUsageFlags,
        properties: vulkanalia::vk::MemoryPropertyFlags,
    ) -> PResult<VulkanBuffer> {
        let buffer_info = vulkanalia::vk::BufferCreateInfo::builder()
            .size(buffer_size)
            .usage(usage)
            .sharing_mode(vulkanalia::vk::SharingMode::EXCLUSIVE);
    
        let buffer = unsafe { vk_device.create_buffer(&buffer_info, None)? };
    
        let requirements = unsafe { vk_device.get_buffer_memory_requirements(buffer) };
    
        let memory_info = vulkanalia::vk::MemoryAllocateInfo::builder()
            .allocation_size(requirements.size)
            .memory_type_index(Self::get_memory_type_index(
                vk_instance,
                vk_physical_device,
                properties,
                requirements,
            )?);
    
        let memory = unsafe { vk_device.allocate_memory(&memory_info, None)? };
    
        unsafe { vk_device.bind_buffer_memory(buffer, memory, 0)?; }
    
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

    /// Map the buffer memory to the CPU and copy data to it.
    /// This is similar to calling map, write, unmap.
    /// This calls several Vulkan operations, so it is not recommended to use this for frequent updates.
    pub fn map_data<T>(
        &mut self,
        vk_device: &vulkanalia::Device,
        data: &[T],
        offset: usize,
    ) -> PResult<()> {
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

    /// Map the buffer memory to the CPU.
    /// SAFETY : The buffer MUST be unmapped after use.
    pub fn map(&mut self, vk_device: &vulkanalia::Device) -> PResult<*mut std::ffi::c_void> {
        let memory = unsafe { vk_device.map_memory(
            self.memory,
            0,
            self.buffer_size,
            vulkanalia::vk::MemoryMapFlags::empty(),
        )? };

        Ok(memory)
    }

    /// Write data to the buffer memory.
    /// SAFETY : The buffer MUST be mapped before use, and the provided pointer must be valid.
    pub fn write<T>(
        &mut self,
        mem_ptr: *mut std::ffi::c_void,
        data: &[T],
    ) {
        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), mem_ptr.cast(), data.len());
        }
    }

    /// Unmap the buffer memory from the CPU.
    /// SAFETY : The buffer MUST be mapped before use.
    pub fn unmap(&mut self, vk_device: &vulkanalia::Device) {
        unsafe {
            vk_device.unmap_memory(self.memory);
        }
    }

    pub fn buffer_info(&self) -> vulkanalia::vk::DescriptorBufferInfoBuilder {
        vulkanalia::vk::DescriptorBufferInfo::builder()
            .buffer(self.buffer)
            .offset(0)
            .range(self.buffer_size)
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
        if PROPELLANT_DEBUG_FEATURES {
            // check if the buffer is none null
            if self.buffer.is_null() {
                println!("[PROPELLANT DEBUG] Attempt to free a null buffer");
                return;
            }
        }
        unsafe {
            vk_device.destroy_buffer(self.buffer, None);
            vk_device.free_memory(self.memory, None);
        }
    }
}
