use std::rc::Rc;

use foundry::ComponentTable;
use crate::engine::{window::vulkan::vulkan_buffer::VulkanBuffer, errors::PropellantError};

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

use super::per_frame_uniform_builder::PerFrameUniformBuilder;

/// Packs up a per-frame uniform, built from any struct.
pub struct PerFrameUniformObject {
    /// A function to generate the object and upload it to the buffer.
    object_getter: Rc<dyn Fn(
        &vulkanalia::Device,
        usize,
        &mut ComponentTable,
        &mut VulkanBuffer,
    ) -> Result<(), PropellantError>>,
    /// The vulkan buffer to upload the uniform to.
    buffer: VulkanBuffer,
    /// The descriptor set layout, a blueprint on how the descriptor set matches the shader.
    layout: vulkanalia::vk::DescriptorSetLayout,
    /// The descriptor sets, one for each swapchain image.
    sets: Vec<vulkanalia::vk::DescriptorSet>,
}

impl PerFrameUniformObject {
    /// Creates a new per frame uniform object, from a function that can generate our uniform object from the components
    /// (per frame uniforms are built from the whole comp table)
    /// and from the vk instance, device etc (for the buffer creation)
    pub fn new(
        builder: &PerFrameUniformBuilder,
        instance: &vulkanalia::Instance,
        device: &vulkanalia::Device,
        physical_device: vulkanalia::vk::PhysicalDevice,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
    ) -> Result<PerFrameUniformObject, PropellantError> {
        // create the vulkan buffer
        let buffer = VulkanBuffer::create(
            instance, device, physical_device,
            (builder.object_size() * swapchain_images_count) as u64,
            vulkanalia::vk::BufferUsageFlags::UNIFORM_BUFFER,
            vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE
                | vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // create the descriptor set layout
        // the layout is a blueprint on how the descriptor set matches the shader.
        let layout_builder = vulkanalia::vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1) 
            .stage_flags(vulkanalia::vk::ShaderStageFlags::VERTEX);

        let bindings = &[layout_builder];
        
        let info = vulkanalia::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings);
        
        let layout = unsafe { device.create_descriptor_set_layout(&info, None)? };

        let object_size = builder.object_size() as u64;

        let sets = Self::create_descriptor_set(
            device,
            descriptor_pool,
            swapchain_images_count,
            layout,
            &buffer,
            object_size,
        )?;

        let object_getter = builder.builder();

        Ok(PerFrameUniformObject {
            object_getter,
            buffer,
            layout,
            sets,
        })
    }

    /// update the buffer value.
    pub fn update_buffer(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        components: &mut ComponentTable,
    ) -> Result<(), PropellantError> {
        (self.object_getter)(vk_device, image_index, components, &mut self.buffer)
    }

    /// Create our descriptor sets from the given pool.
    /// The pool might overflow, so in the future we should look into reallocating the pool.
    /// Creation would usually be done once at the start of the app.
    fn create_descriptor_set(
        vk_device: &vulkanalia::Device,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
        layout: vulkanalia::vk::DescriptorSetLayout,
        buffer: &VulkanBuffer,
        object_size: u64,
    ) -> Result<Vec<vulkanalia::vk::DescriptorSet>, PropellantError> {
        // create one descriptor set per swapchain image.
        let layouts = vec![layout; swapchain_images_count];
        let info = vulkanalia::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let sets = unsafe { vk_device.allocate_descriptor_sets(&info)? };

        // populate the descriptor sets.
        for i in 0..swapchain_images_count {
            let info = vulkanalia::vk::DescriptorBufferInfo::builder()
                .buffer(buffer.buffer())
                .offset(0)
                .range(object_size);

            let buffer_info = &[info];

            let ubo_write = vulkanalia::vk::WriteDescriptorSet::builder()
                .dst_set(sets[i])
                .dst_binding(0)
                .dst_array_element(0) // this is the element we can change for huge buffers ?
                .descriptor_type(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            unsafe { 
                vk_device.update_descriptor_sets(&[ubo_write], &[] as &[vulkanalia::vk::CopyDescriptorSet]);
            }
        }

        Ok(sets)
    }

    /// Get the descriptor set for the given image index.
    pub fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet {
        self.sets[image_index]
    }

    /// get the layout.
    pub fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout
    }

    /// clear the ressources used by this object.
    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        unsafe {
            vk_device.destroy_descriptor_set_layout(self.layout, None);
            self.buffer.destroy(vk_device);
        }
    }
}

