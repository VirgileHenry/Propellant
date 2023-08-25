use std::{ffi::c_void, fmt::Debug};

use crate::engine::consts::PROPELLANT_DEBUG_FEATURES;
use crate::engine::{window::vulkan::vulkan_buffer::VulkanBuffer, errors::PResult};

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;

/// Wrapper around a uniform of type T.
#[derive(Debug)]
pub struct UniformBufferBuilder<T> {
    phantom: std::marker::PhantomData<T>,
    stage: vulkanalia::vk::ShaderStageFlags,
    descriptor_type: vulkanalia::vk::DescriptorType,
}

impl<T: Debug + 'static> UniformBufferBuilder<T> {
    /// Creates a new empty uniform buffer.
    pub fn new(
        stage: vulkanalia::vk::ShaderStageFlags,
        descriptor_type: vulkanalia::vk::DescriptorType,
    ) -> UniformBufferBuilder<T> {
        UniformBufferBuilder {
            phantom: std::marker::PhantomData,
            stage,
            descriptor_type,
        }
    }

    pub fn build(
        &self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_image_count: usize,
    ) -> PResult<UniformBuffer<T>> {
        UniformBuffer::new(
            self,
            vk_device,
            vk_descriptor_pool,
            swapchain_image_count,
        )
    }

    pub fn stage(&self) -> vulkanalia::vk::ShaderStageFlags {
        self.stage
    }

    pub fn descriptor_type(&self) -> vulkanalia::vk::DescriptorType {
        self.descriptor_type
    }
}


#[derive(Debug)]
enum UniformBufferMemoryState {
    /// The buffer is mapped, and we carry the pointer to the mapped memory.
    Mapped(*mut c_void),
    /// The buffer is currently not mapped.
    AtRest,
    /// The buffer have not been initialized yet.
    Uninitialized,
}

#[derive(Debug)]
pub struct UniformBuffer<T> {
    phantom: std::marker::PhantomData<T>,
    /// The state of our vk buffer, if it is mapped or not.
    buffer_state: UniformBufferMemoryState,
    /// DS layout
    layout: vulkanalia::vk::DescriptorSetLayout,
    /// descriptor sets and buffers for each frame.
    sets_and_buffers: Vec<(vulkanalia::vk::DescriptorSet, VulkanBuffer)>,
    /// The type of descriptor buffer we have : uniform for per frame, storage for per object.
    descriptor_type: vulkanalia::vk::DescriptorType,
}

impl<T: Debug + 'static> UniformBuffer<T> {
    pub fn new(
        builder: &UniformBufferBuilder<T>,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_image_count: usize,
    ) -> PResult<UniformBuffer<T>> {
        // create the descriptor set layout
        // the layout is a blueprint on how the descriptor set matches the shader.
        let layout_binding_builder = vulkanalia::vk::DescriptorSetLayoutBinding::builder()
            .binding(0) // all zero for now, maybe this will change ?
            .descriptor_type(builder.descriptor_type())
            .descriptor_count(1) 
            .stage_flags(builder.stage());

        let layout_bindings = &[layout_binding_builder];
        
        let info = vulkanalia::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(layout_bindings);
        
        let layout = unsafe { vk_device.create_descriptor_set_layout(&info, None)? };

        let descriptor_sets = Self::create_descriptor_set(
            vk_device,
            vk_descriptor_pool,
            swapchain_image_count,
            layout,
        )?;

        Ok(UniformBuffer {
            buffer_state: UniformBufferMemoryState::Uninitialized,
            phantom: std::marker::PhantomData,
            layout,
            sets_and_buffers: descriptor_sets.into_iter().map(|ds| (ds, VulkanBuffer::empty())).collect(),
            descriptor_type: builder.descriptor_type(),
        })
    }

    /// Maps the memory of the buffer to the CPU, to be able to write to it.
    /// This is a vulkan operation.
    pub fn map(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
    ) -> PResult<()> {
        // this could be an optimisation point : 
        // if we are sure the memory is at rest, we can directly map it.
        match self.buffer_state {
            UniformBufferMemoryState::AtRest => self.buffer_state = UniformBufferMemoryState::Mapped(self.sets_and_buffers[image_index].1.map(vk_device)?),
            _ => {/* buffer might be of size 0 and uninit, it's ok */}
        }
        Ok(())
    }

    pub fn update_buffer(
        &mut self,
        buffer_index: usize,
        image_index: usize,
        data: T, // maybe &[T] ?
    ) {
        // compute the offset in bytes
        let byte_offset = std::mem::size_of::<T>() * buffer_index;

        // write to the buffer, offseted. As the size of c_void is 1 byte, the byte offset is indeed in bytes.
        // otherwise, be careful as the add() function offset of offset * size_of::<T>().
        match self.buffer_state {
            UniformBufferMemoryState::Mapped(mem) => self.sets_and_buffers[image_index].1.write(
                unsafe {mem.add(byte_offset)},
                std::slice::from_ref(&data)
            ),
            _ => {/* buffer might be of size 0 and uninit, it's ok */}
        }
    }

    /// Unmaps the memory of the buffer from the CPU, to be able to write to it.
    /// This is a vulkan operation.
    pub fn unmap(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
    ) {
        match self.buffer_state {
            UniformBufferMemoryState::Mapped(_) => {
                self.sets_and_buffers[image_index].1.unmap(vk_device);
                self.buffer_state = UniformBufferMemoryState::AtRest;
            }
            _ => {/* buffer might be of size 0 and uninit, it's ok */}
        }
    }
    
    /// Create our descriptor sets from the given pool.
    /// The pool might overflow, so in the future we should look into reallocating the pool.
    /// Creation would usually be done once at the start of the app.
    fn create_descriptor_set(
        vk_device: &vulkanalia::Device,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_image_count: usize,
        layout: vulkanalia::vk::DescriptorSetLayout,
    ) -> PResult<Vec<vulkanalia::vk::DescriptorSet>> {
        // create one descriptor set per swapchain image.
        let layouts = vec![layout; swapchain_image_count];
        let info = vulkanalia::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let sets = unsafe { vk_device.allocate_descriptor_sets(&info)? };

        Ok(sets)
    }

    pub fn populate_descriptor_set(
        &self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        object_count: usize,
    ) -> PResult<()> {
        // the buffer info points to our buffer, offseted to match the frame buffer.
        let info = vulkanalia::vk::DescriptorBufferInfo::builder()
            .buffer(self.sets_and_buffers[image_index].1.buffer())
            .offset(0)
            .range((object_count * std::mem::size_of::<T>()) as u64);

        let buffer_info = &[info];

        let ubo_write = vulkanalia::vk::WriteDescriptorSet::builder()
            .dst_set(self.sets_and_buffers[image_index].0)
            .dst_binding(0)
            .dst_array_element(0) 
            .descriptor_type(self.descriptor_type)
            .buffer_info(buffer_info);

        unsafe { 
            vk_device.update_descriptor_sets(&[ubo_write], &[] as &[vulkanalia::vk::CopyDescriptorSet]);
        }
        Ok(())
    }

    /// Assert that our buffer is big enough to hold all the objects.
    /// If the buffer is too small, it will be reallocated.
    pub fn assert_buffer_size(
        &mut self,
        object_count: usize,
        image_index: usize,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
    ) -> PResult<()> {
        let buffer_size = (object_count * std::mem::size_of::<T>()) as u64;
        
        if buffer_size > self.sets_and_buffers[image_index].1.size() {

            if PROPELLANT_DEBUG_FEATURES {
                if let UniformBufferMemoryState::Mapped(_) = self.buffer_state {
                    panic!("[PROPELLANT DEBUG] Reallocating buffer called on a buffer that is mapped.");
                }
            }

            // the buffer is no longer unititialized if it was
            self.buffer_state = UniformBufferMemoryState::AtRest;

            // destroy the previous buffer
            if self.sets_and_buffers[image_index].1.size() > 0 {
                self.sets_and_buffers[image_index].1.destroy(vk_device);
            }
            // reallocate the buffer
            let usage = match self.descriptor_type {
                vulkanalia::vk::DescriptorType::UNIFORM_BUFFER => vulkanalia::vk::BufferUsageFlags::UNIFORM_BUFFER,
                vulkanalia::vk::DescriptorType::STORAGE_BUFFER => vulkanalia::vk::BufferUsageFlags::STORAGE_BUFFER,
                _ => panic!("[PROPELLANT ERROR] Invalid descriptor type for UniformBuffer: not uniform nor storage buffer.\nIf this is intended, please add the new usage case here."),
            };
            self.sets_and_buffers[image_index].1 = VulkanBuffer::create(
                vk_instance,
                vk_device,
                vk_physical_device,
                buffer_size,
                usage,
                vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE | vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            // repopulate ds
            self.populate_descriptor_set(vk_device, image_index, object_count)?;
        }
        Ok(())
    }

    pub fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet {
        self.sets_and_buffers[image_index].0
    }

    pub fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout
    }

    pub fn destroy_buffer(&mut self, vk_device: &vulkanalia::Device) {
        self.sets_and_buffers.iter_mut().for_each(|(_, buffer)| {
            if buffer.size() > 0 {
                buffer.destroy(vk_device);
            }
        });
        unsafe {
            vk_device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}