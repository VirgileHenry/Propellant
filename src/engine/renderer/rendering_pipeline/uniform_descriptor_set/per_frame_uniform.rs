use std::rc::Rc;

use foundry::ComponentTable;
use crate::engine::errors::PResult;
use crate::engine::window::vulkan::vulkan_buffer::VulkanBuffer;

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

use super::per_frame_uniform_builder::PerFrameUniformBuilder;
use super::uniform_update_frequency::UniformUpdateFrequency;

/// Packs up a per-frame uniform, built from any struct.
pub struct PerFrameUniformObject {
    /// A function to generate the object and upload it to the buffer.
    buffer_update: Rc<dyn Fn(
        &vulkanalia::Device,
        usize,
        &ComponentTable,
        &mut VulkanBuffer,
    ) -> PResult<()>>,
    /// The vulkan buffer to upload the uniform to.
    buffer: VulkanBuffer,
    /// The descriptor set layout, a blueprint on how the descriptor set matches the shader.
    layout: vulkanalia::vk::DescriptorSetLayout,
    /// The descriptor sets, one for each swapchain image.
    sets: Vec<vulkanalia::vk::DescriptorSet>,
    /// The update frequency of the uniform.
    update_frequency: UniformUpdateFrequency,
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
    ) -> PResult<PerFrameUniformObject> {
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
            .binding(builder.binding())
            .descriptor_type(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1) 
            .stage_flags(builder.stage());

        let bindings = &[layout_builder];
        
        let info = vulkanalia::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings);
        
        let layout = unsafe { device.create_descriptor_set_layout(&info, None)? };

        let object_size = builder.object_size();

        let sets = Self::create_descriptor_set(
            device,
            descriptor_pool,
            swapchain_images_count,
            layout,
            &buffer,
            object_size,
        )?;

        Ok(PerFrameUniformObject {
            buffer_update: builder.buffer_update(),
            buffer,
            layout,
            sets,
            update_frequency: builder.update_frequency().clone(),
        })
    }

    /// update the buffer value.
    pub fn update_buffer(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        components: &mut ComponentTable,
        delta_time: f32,
    ) -> PResult<()> {
        // update the buffer based on the update frequency
        match &mut self.update_frequency {
            // if we update he buffer every frame, just update it.
            UniformUpdateFrequency::EachFrame => (self.buffer_update)(vk_device, image_index, components, &mut self.buffer),
            // is we only update the buffer at the start, do not update it.
            UniformUpdateFrequency::StartOnly => Ok(()),
            // if the buffer have a fixed time update, increase time and check for update.
            // we need to keep a timer for each image in flight, so we do update every part of the buffer.
            UniformUpdateFrequency::Timed(time_vec, rate) => {
                // assert the vec is big enough, other wise expand it
                // at some point this will match the number of swapchain images, and we won't expand it anymore
                // that's why we expand of exactly one object, to not over extend.
                while time_vec.len() < image_index + 1 {
                    time_vec.reserve_exact(1);
                    time_vec.push(time_vec[0]);
                }
                // increase the time on all timers.
                time_vec.iter_mut().for_each(|t| *t += delta_time);
                // if the timer of current image is bigger than the rate, update the buffer and reset the timer for this image.
                if time_vec[image_index] >= *rate {
                    // first a if to detect, than a while : maybe overkill and too much overhead, 
                    // but this does not update the buffer multiple times.
                    while time_vec[image_index] > *rate {
                        time_vec[image_index] -= *rate;
                    }
                    (self.buffer_update)(vk_device, image_index, components, &mut self.buffer)
                }
                else {
                    // if we should not update the buffer, just return ok.
                    Ok(())
                }
            }
        }
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
        object_size: usize,
    ) -> PResult<Vec<vulkanalia::vk::DescriptorSet>> {
        // create one descriptor set per swapchain image.
        let layouts = vec![layout; swapchain_images_count];
        let info = vulkanalia::vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let sets = unsafe { vk_device.allocate_descriptor_sets(&info)? };

        // populate the descriptor sets.
        Self::populate_descriptor_sets(
            vk_device,
            swapchain_images_count,
            buffer,
            object_size,
            &sets
        )?;

        Ok(sets)
    }

    fn populate_descriptor_sets(
        vk_device: &vulkanalia::Device,
        swapchain_images_count: usize,
        buffer: &VulkanBuffer,
        object_size: usize,
        sets: &Vec<vulkanalia::vk::DescriptorSet>,
    ) -> PResult<()> {
        // populate the descriptor sets.
        for i in 0..swapchain_images_count {
            let info = vulkanalia::vk::DescriptorBufferInfo::builder()
                .buffer(buffer.buffer())
                .offset(0)
                .range(object_size as u64);

            let buffer_info = &[info];

            let ubo_write = vulkanalia::vk::WriteDescriptorSet::builder()
                .dst_set(sets[i])
                .dst_binding(0)
                .dst_array_element(0) // this is the element we can change for huge buffers ?
                .descriptor_type(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(buffer_info);

            // todo : factorize this into a single operation
            unsafe { 
                vk_device.update_descriptor_sets(&[ubo_write], &[] as &[vulkanalia::vk::CopyDescriptorSet]);
            }
        }

        Ok(())
    }

    /// Get the descriptor set for the given image index.
    pub fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet {
        self.sets[image_index]
    }

    /// get the layout.
    pub fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout
    }

    pub fn update_frequency(&self) -> &UniformUpdateFrequency {
        &self.update_frequency
    }

    /// clear the ressources used by this object.
    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        unsafe {
            vk_device.destroy_descriptor_set_layout(self.layout, None);
            self.buffer.destroy(vk_device);
        }
    }
}

