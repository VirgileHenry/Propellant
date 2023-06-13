use std::rc::Rc;

use crate::engine::errors::PResult;
use crate::{Transform, Material};
use crate::engine::window::vulkan::vulkan_buffer::VulkanBuffer;

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

use super::per_object_uniform_builder::PerObjectUniformBuilder;
use super::uniform_update_frequency::UniformUpdateFrequency;

/// Packs up a per-object uniform, built from any struct.
pub struct PerObjectUniformObject {
    /// A function to generate the object and upload it to the buffer.
    buffer_update: Rc<dyn Fn(
        &vulkanalia::Device, // vk_device
        usize, // image_index
        usize, // image_count
        &Transform, // transform
        &Material, // material
        usize, // offset
        &mut VulkanBuffer, // target buffer
    ) -> PResult<()>>,
    /// Size of the inner object used as uniform. Usefull for buffer recreation.
    object_size: usize,
    /// The vulkan buffer to upload the uniform to.
    buffer: VulkanBuffer,
    /// The descriptor set layout, a blueprint on how the descriptor set matches the shader.
    layout: vulkanalia::vk::DescriptorSetLayout,
    /// The descriptor sets, one for each swapchain image.
    sets: Vec<vulkanalia::vk::DescriptorSet>,
    /// The update frequency of the uniform.
    update_frequency: UniformUpdateFrequency,
}

impl PerObjectUniformObject {
    /// Creates a new per object uniform object, from a function that can generate our uniform object from a transform and the materials.
    /// Most per object data that is passed to the shader should be passed by the material, the transform is here to give model matrix.
    pub fn new(
        builder: &PerObjectUniformBuilder,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
    ) -> PResult<PerObjectUniformObject> {
        // todo : this will create a too small buffer, it needs to be recreated from the components.
        // create the vulkan buffer
        let buffer = VulkanBuffer::create(
            vk_instance, vk_device, vk_physical_device,
            1, // size of 0 is an invalid usage.
            vulkanalia::vk::BufferUsageFlags::UNIFORM_BUFFER,
            vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE
                | vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // create the descriptor set layout
        // the layout is a blueprint on how the descriptor set matches the shader.
        let layout_builder = vulkanalia::vk::DescriptorSetLayoutBinding::builder()
            .binding(builder.binding())
            .descriptor_type(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .descriptor_count(1) 
            .stage_flags(builder.stage());

        let bindings = &[layout_builder];
        
        let info = vulkanalia::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings);
        
        let layout = unsafe { vk_device.create_descriptor_set_layout(&info, None)? };

        let object_size = builder.object_size();

        let sets = Self::create_descriptor_set(
            vk_device,
            vk_descriptor_pool,
            swapchain_images_count,
            layout,
            &buffer,
            object_size,
        )?;

        let buffer_update = builder.buffer_update();

        Ok(PerObjectUniformObject {
            buffer_update,
            buffer,
            object_size,
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
        image_count: usize,
        transform: &Transform,
        material: &Material,
        offset: usize,
        delta_time: f32,
    ) -> PResult<()> {
        // update the buffer based on the update frequency
        match &mut self.update_frequency {
            // if we update he buffer every frame, just update it.
            UniformUpdateFrequency::EachFrame => (self.buffer_update)(vk_device, image_index, image_count, transform, material, offset, &mut self.buffer),
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
                    (self.buffer_update)(vk_device, image_index, image_count, transform, material, offset, &mut self.buffer)
                }
                else {
                    // if we should not update the buffer, just return ok.
                    Ok(())
                }
            }
        }
    }

    /// update the buffer value.
    /// Returns a boolean telling wether or not the buffer needs to be recomputed.
    /// The buffer will need to be recomputed if an entity is not in the map, meaning the scene geometry changed.
    pub fn update_start_only_buffer(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_count: usize,
        transform: &Transform,
        material: &Material,
        offset: usize,
    ) -> PResult<()> {
        match self.update_frequency {
            UniformUpdateFrequency::StartOnly => {
                // a little weird, as we are updating the buffer for all images, so some might be in flight ?
                // but for now, let's consider the start only uniforms are not too affetced by this !
                for i in 0..image_count {
                    (self.buffer_update)(vk_device, i, image_count, transform, material, offset, &mut self.buffer)?;
                }
                Ok(())
            },
            _ => Ok(()),
        }
    }

    pub fn resize_buffer(
        &mut self, 
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        swapchain_images_count: usize,
        entity_count: usize,
    ) -> PResult<()> {
        let new_size = entity_count * swapchain_images_count * self.object_size;
        if self.buffer.size() < new_size as u64 {
            // free the previous buffer
            self.buffer.destroy(vk_device);
            // allocate the new buffer
            self.buffer = VulkanBuffer::create(
                vk_instance,
                vk_device,
                vk_physical_device,
                new_size as u64,
                vulkanalia::vk::BufferUsageFlags::UNIFORM_BUFFER,
                vulkanalia::vk::MemoryPropertyFlags::HOST_VISIBLE
                    | vulkanalia::vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            // now, we need to repopulate the descriptor sets so they point to the good buffer.
            Self::populate_descriptor_sets(
                vk_device,
                swapchain_images_count,
                &self.buffer,
                new_size,
                &self.sets,
            )?;
        }

        Ok(())
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
        Self::populate_descriptor_sets(vk_device, swapchain_images_count, buffer, object_size, &sets)?;

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
                .dst_binding(2)
                .dst_array_element(0) 
                .descriptor_type(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .buffer_info(buffer_info);

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

    pub fn byte_offset(&self, offset: usize, image_index: usize, swapchain_image_count: usize) -> usize {
        self.object_size * offset * swapchain_image_count + self.object_size * image_index
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

