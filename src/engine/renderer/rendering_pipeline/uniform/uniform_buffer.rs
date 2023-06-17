use std::{ffi::c_void, fmt::Debug};

use crate::engine::{window::vulkan::vulkan_buffer::VulkanBuffer, errors::PResult};

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;

/// Wrapper around a uniform of type T.
#[derive(Debug)]
pub struct UniformBufferBuilder<T> {
    phantom: std::marker::PhantomData<T>,
    stage: vulkanalia::vk::ShaderStageFlags,
    binding: u32,
}

impl<T: Debug + 'static> UniformBufferBuilder<T> {
    /// Creates a new empty uniform buffer.
    pub fn new(
        stage: vulkanalia::vk::ShaderStageFlags,
        binding: u32,
    ) -> UniformBufferBuilder<T> {
        UniformBufferBuilder {
            phantom: std::marker::PhantomData,
            stage,
            binding,
        }
    }

    pub fn build(
        &self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
        descriptor_type: vulkanalia::vk::DescriptorType,
    ) -> PResult<UniformBuffer<T>> {
        UniformBuffer::new(
            self,
            vk_device,
            vk_descriptor_pool,
            swapchain_images_count,
            self.binding,
            descriptor_type,
        )
    }

    pub fn stage(&self) -> vulkanalia::vk::ShaderStageFlags {
        self.stage
    }

    pub fn binding(&self) -> u32 {
        self.binding
    }
}


#[derive(Debug)]
enum UniformBufferMemoryState {
    Mapped(*mut c_void),
    Unmapped,
}

#[derive(Debug)]
pub struct UniformBuffer<T> {
    phantom: std::marker::PhantomData<T>,
    /// The allocated vulkan buffer containing all the uniform data.
    /// It is duplicated for each frame in flight.
    /// [ frame 1 obj1 | frame 1 obj2 | ... | frame 1 objn | frame 2 obj1 | ... | frame n objn]
    vk_buffer: VulkanBuffer,
    /// The state of our vk buffer, if it is mapped or not.
    buffer_state: UniformBufferMemoryState,
    /// DS layout
    layout: vulkanalia::vk::DescriptorSetLayout,
    /// descriptor sets
    descriptor_sets: Vec<vulkanalia::vk::DescriptorSet>,
    /// the binding at the creation of the buffer. The binding is given by the moment it have been registered in the pipeline.
    binding: u32,
    /// The type of descriptor buffer we have : uniform for per frame, storage for per object.
    descriptor_type: vulkanalia::vk::DescriptorType,
}

impl<T: Debug + 'static> UniformBuffer<T> {
    pub fn new(
        builder: &UniformBufferBuilder<T>,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
        binding: u32,
        descriptor_type: vulkanalia::vk::DescriptorType,
    ) -> PResult<UniformBuffer<T>> {
        // create the descriptor set layout
        // the layout is a blueprint on how the descriptor set matches the shader.
        let layout_builder = vulkanalia::vk::DescriptorSetLayoutBinding::builder()
            .binding(builder.binding())
            .descriptor_type(descriptor_type)
            .descriptor_count(1) 
            .stage_flags(builder.stage());

        let bindings = &[layout_builder];
        
        let info = vulkanalia::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings);
        
        let layout = unsafe { vk_device.create_descriptor_set_layout(&info, None)? };

        let descriptor_sets = Self::create_descriptor_set(
            vk_device,
            vk_descriptor_pool,
            swapchain_images_count,
            layout,
        )?;

        Ok(UniformBuffer {
            vk_buffer: VulkanBuffer::empty(),
            buffer_state: UniformBufferMemoryState::Unmapped,
            phantom: std::marker::PhantomData,
            layout,
            descriptor_sets,
            binding,
            descriptor_type,
        })
    }

    /// Maps the memory of the buffer to the CPU, to be able to write to it.
    /// This is a vulkan operation.
    pub fn map(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) -> PResult<()> {
        // additional checks in debug mode
        if cfg!(debug_assertions) {
            match self.buffer_state {
                UniformBufferMemoryState::Mapped(_) => {
                    panic!("[PROPELLANT DEBUG] UniformBuffer::map() called on a buffer that is already mapped.");
                }
                UniformBufferMemoryState::Unmapped => {}
            }
        }
        self.buffer_state = UniformBufferMemoryState::Mapped(self.vk_buffer.map(vk_device)?);
        Ok(())
    }

    pub fn update_buffer(
        &mut self,
        instance_id: usize,
        image_index: usize,
        instance_count: usize,
        data: &T, // maybe &[T] ?
    ) {
        // compute the offset in bytes
        let byte_offset = std::mem::size_of::<T>() * instance_count * image_index + std::mem::size_of::<T>() * instance_id;

        // write to the buffer, offseted. As the size of c_void is 1 byte, the byte offset is indeed in bytes.
        // otherwise, be careful as the add() function offset of offset * size_of::<T>().
        match self.buffer_state {
            UniformBufferMemoryState::Mapped(mem) => self.vk_buffer.write(
                unsafe {mem.add(byte_offset)},
                std::slice::from_ref(data)
            ),
            UniformBufferMemoryState::Unmapped => {
                if cfg!(debug_assertions) {
                    panic!("[PROPELLANT DEBUG] UniformBuffer::update_buffer() called on a buffer that is not mapped.");
                }
            }
        }
    }

    /// Unmaps the memory of the buffer from the CPU, to be able to write to it.
    /// This is a vulkan operation.
    pub fn unmap(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        // additional checks in debug mode
        if cfg!(debug_assertions) {
            match self.buffer_state {
                UniformBufferMemoryState::Mapped(_) => {}
                UniformBufferMemoryState::Unmapped => {
                    panic!("[PROPELLANT DEBUG] UniformBuffer::unmap() called on a buffer that is already unmapped.");
                }
            }
        }
        self.vk_buffer.unmap(vk_device);
        self.buffer_state = UniformBufferMemoryState::Unmapped;
    }

    
    /// Create our descriptor sets from the given pool.
    /// The pool might overflow, so in the future we should look into reallocating the pool.
    /// Creation would usually be done once at the start of the app.
    fn create_descriptor_set(
        vk_device: &vulkanalia::Device,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
        layout: vulkanalia::vk::DescriptorSetLayout,
    ) -> PResult<Vec<vulkanalia::vk::DescriptorSet>> {
        // create one descriptor set per swapchain image.
        let layouts = vec![layout; swapchain_images_count];
        let info = vulkanalia::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let sets = unsafe { vk_device.allocate_descriptor_sets(&info)? };

        Ok(sets)
    }

    pub fn populate_descriptor_sets(
        &self,
        vk_device: &vulkanalia::Device,
        swapchain_image_count: usize,
        object_count: usize,
    ) -> PResult<()> {
        // populate the descriptor sets.
        for i in 0..swapchain_image_count {
            // the buffer info points to our buffer, offseted to match the frame buffer.
            let info = vulkanalia::vk::DescriptorBufferInfo::builder()
                .buffer(self.vk_buffer.buffer())
                .offset((object_count * std::mem::size_of::<T>() * i) as u64)
                .range((object_count * std::mem::size_of::<T>()) as u64);

            let buffer_info = &[info];

            let ubo_write = vulkanalia::vk::WriteDescriptorSet::builder()
                .dst_set(self.descriptor_sets[i])
                .dst_binding(self.binding)
                .dst_array_element(0) 
                .descriptor_type(self.descriptor_type)
                .buffer_info(buffer_info);

            unsafe { 
                vk_device.update_descriptor_sets(&[ubo_write], &[] as &[vulkanalia::vk::CopyDescriptorSet]);
            }
        }

        Ok(())
    }

    /// Assert that our buffer is big enough to hold all the objects.
    /// If the buffer is too small, it will be reallocated.
    pub fn assert_buffer_size(
        &mut self,
        object_count: usize,
        swapchain_image_count: usize,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
    ) -> PResult<()> {
        let buffer_size = (object_count * swapchain_image_count * std::mem::size_of::<T>()) as u64;

        if buffer_size > self.vk_buffer.size() {
            // destroy the previous buffer
            if self.vk_buffer.size() > 0 {
                self.vk_buffer.destroy(vk_device);
            }
            // reallocate the buffer
            let usage = match self.descriptor_type {
                vulkanalia::vk::DescriptorType::UNIFORM_BUFFER => vulkanalia::vk::BufferUsageFlags::UNIFORM_BUFFER,
                vulkanalia::vk::DescriptorType::STORAGE_BUFFER => vulkanalia::vk::BufferUsageFlags::STORAGE_BUFFER,
                _ => panic!("[PROPELLANT ERROR] Invalid descriptor type for UniformBuffer: not uniform nor storage buffer.\nIf this is intended, please add the new usage case here."),
            };
            self.vk_buffer = VulkanBuffer::create(
                vk_instance,
                vk_device,
                vk_physical_device,
                buffer_size,
                usage,
                vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE | vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            // repopulate ds
            self.populate_descriptor_sets(vk_device, swapchain_image_count, object_count)?;
        }
        Ok(())
    }

    pub fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet {
        self.descriptor_sets[image_index]
    }

    pub fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout
    }

    pub fn destroy_buffer(&mut self, vk_device: &vulkanalia::Device) {
        if self.vk_buffer.size() > 0 {
            self.vk_buffer.destroy(vk_device);
        }
        unsafe {
            vk_device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}