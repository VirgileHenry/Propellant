use foundry::ComponentTable;
use crate::engine::{window::vulkan::vulkan_buffer::VulkanBuffer, errors::PropellantError};

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

pub struct PerFrameUniformObject {
    object: Box<dyn Fn(
        &vulkanalia::Device,
        usize,
        &mut ComponentTable,
        &mut VulkanBuffer,
    ) -> Result<(), PropellantError>>,
    buffer: VulkanBuffer,
    layout: vulkanalia::vk::DescriptorSetLayout,
    sets: Vec<vulkanalia::vk::DescriptorSet>,
}

impl PerFrameUniformObject {
    pub fn new<T: 'static>(
        uniform_object_creator: fn(&ComponentTable) -> Result<T, PropellantError>,
        instance: &vulkanalia::Instance,
        device: &vulkanalia::Device,
        physical_device: vulkanalia::vk::PhysicalDevice,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
    ) -> Result<PerFrameUniformObject, PropellantError> {
        // create the vulkan buffer
        let buffer = VulkanBuffer::create(
            instance, device, physical_device,
            (std::mem::size_of::<T>() * swapchain_images_count) as u64,
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

        // create the function that generate the object and upload it to the buffer.
        let object = Box::new(move |
            vk_device: &vulkanalia::Device,
            image_index: usize,
            components: &mut ComponentTable,
            buffer: &mut VulkanBuffer,
        | {
            uniform_object_creator(components).and_then(
                |object| buffer.map_data(vk_device, &[object], image_index * std::mem::size_of::<T>())
            )
        });

        let object_size = std::mem::size_of::<T>() as u64;

        let sets = Self::create_descriptor_set(
            device,
            descriptor_pool,
            swapchain_images_count,
            layout,
            &buffer,
            object_size,
        )?;

        Ok(PerFrameUniformObject {
            object,
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
        (self.object)(vk_device, image_index, components, &mut self.buffer)
    }

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

