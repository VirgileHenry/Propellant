use std::rc::Rc;

use foundry::ComponentTable;
use crate::engine::errors::PResult;
use crate::engine::window::vulkan::vulkan_buffer::VulkanBuffer;

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
    /// The update frequency of the uniform.
    update_frequency: UniformUpdateFrequency,
    /// the binding for this uniform.
    binding: u32,
    /// The shader to stage to update the uniform to.
    stage: vulkanalia::vk::ShaderStageFlags,
}

impl PerFrameUniformObject {
    /// Creates a new per frame uniform object, from a function that can generate our uniform object from the components
    /// (per frame uniforms are built from the whole comp table)
    /// and from the vk instance, device etc (for the buffer creation)
    pub fn build(
        builder: &PerFrameUniformBuilder,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        swapchain_images_count: usize,
    ) -> PResult<PerFrameUniformObject> {
        // create the vulkan buffer that will store the uniform.
        let buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            (builder.object_size() * swapchain_images_count) as u64,
            vulkanalia::vk::BufferUsageFlags::UNIFORM_BUFFER,
            vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE
                | vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        Ok(PerFrameUniformObject {
            buffer_update: builder.buffer_update(),
            buffer,
            update_frequency: builder.update_frequency().clone(),
            binding: builder.binding(),
            stage: builder.stage(),
        })
    }

    pub fn layout(&self) -> <vulkanalia::vk::DescriptorSetLayoutBinding as vulkanalia::vk::HasBuilder>::Builder {
        vulkanalia::vk::DescriptorSetLayoutBinding::builder()
            .binding(self.binding)
            .descriptor_type(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1) 
            .stage_flags(self.stage)
    }

    pub fn buffer_info(&self) -> vulkanalia::vk::DescriptorBufferInfoBuilder {
        self.buffer.buffer_info()
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

    pub fn binding(&self) -> u32 {
        self.binding
    }

    /// clear the ressources used by this object.
    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.buffer.destroy(vk_device);
    }
}

