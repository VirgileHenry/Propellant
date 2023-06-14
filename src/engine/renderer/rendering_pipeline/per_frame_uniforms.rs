use foundry::ComponentTable;
use crate::engine::errors::PResult;
use super::uniform_descriptor_set::per_frame_uniform::PerFrameUniformObject;
use super::uniform_descriptor_set::per_frame_uniform_builder::PerFrameUniformBuilder;

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;


pub struct PerFrameUniforms {
    /// the descriptors and buffers for each uniforms.
    uniforms: Vec<PerFrameUniformObject>,
    /// The descriptor set layout, a blueprint on how the descriptor set matches the shader.
    layout: vulkanalia::vk::DescriptorSetLayout,
    /// The descriptor sets, one for each swapchain image.
    sets: Vec<vulkanalia::vk::DescriptorSet>,
}

impl PerFrameUniforms {
    pub fn build(
        uniforms: &[PerFrameUniformBuilder],
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
    ) -> PResult<PerFrameUniforms> {
        // build every uniforms that we got
        let uniforms = uniforms.into_iter()
            .map(|builder| PerFrameUniformObject::build(builder, vk_instance, vk_device, vk_physical_device, swapchain_images_count))
            .collect::<PResult<Vec<_>>>()?;

        // for each uniform, get it's descriptor layout to put in one descriptor set !
        let layouts = uniforms.iter()
            .map(|uo| uo.layout())
            .collect::<Vec<_>>();

        // create the descriptor set layout
        let info = vulkanalia::vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layouts);
        
        let layout = unsafe { vk_device.create_descriptor_set_layout(&info, None)? };

        let sets = Self::create_descriptor_set(
            &uniforms,
            vk_device,
            descriptor_pool,
            swapchain_images_count,
            layout,
        )?;

        Ok(PerFrameUniforms {
            uniforms,
            layout,
            sets,
        })
    }

    pub fn bind(
        &self,
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        layout: vulkanalia::vk::PipelineLayout,
        image_index: usize,
    ) {
        // bind the ds of the frame
        let set = [self.sets[image_index]];
        unsafe {
            vk_device.cmd_bind_descriptor_sets(
                command_buffer,
                vulkanalia::vk::PipelineBindPoint::GRAPHICS,
                layout,
                0,
                &set,
                &[],
            );
        }
    }

    pub fn update(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        components: &mut ComponentTable,
        delta_time: f32,
    ) -> PResult<()> {
        // per frame uniforms
        for uniform_object in self.uniforms.iter_mut() {
            uniform_object.update_buffer(vk_device, image_index, components, delta_time)?;
        }

        Ok(())
    }

        /// Create our descriptor sets from the given pool.
    /// The pool might overflow, so in the future we should look into reallocating the pool.
    /// Creation would usually be done once at the start of the app.
    fn create_descriptor_set(
        uniforms: &Vec<PerFrameUniformObject>,
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

        // populate the descriptor sets.
        Self::populate_descriptor_sets(
            uniforms,
            vk_device,
            swapchain_images_count,
            &sets
        )?;

        Ok(sets)
    }

    fn populate_descriptor_sets(
        uniforms: &Vec<PerFrameUniformObject>,
        vk_device: &vulkanalia::Device,
        swapchain_images_count: usize,
        sets: &Vec<vulkanalia::vk::DescriptorSet>,
    ) -> PResult<()> {
        // populate the descriptor sets.
        // for each uniform, get a buffer and layout info.
        // todo : regroup the write, and do only one update descriptor set op.
        let _ds_writes = (0..swapchain_images_count).map(|i| {
            uniforms.iter()
                .map(|uniform| {
                    // for each swap chain image, and for each descriptor, create a write descriptor.
                    // get the buffer info
                    let buf_info = uniform.buffer_info();
                    let info = [buf_info];
                    // create the write descriptor
                    let write = vulkanalia::vk::WriteDescriptorSet::builder()
                        .dst_set(sets[i])
                        .dst_binding(uniform.binding())
                        .dst_array_element(0)
                        .descriptor_type(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(&info);

                    unsafe { 
                        vk_device.update_descriptor_sets(&[write], &[] as &[vulkanalia::vk::CopyDescriptorSet]);
                    }

                    // todo : return the write here !
                }).collect::<Vec<_>>()
                // collect all the write descriptors in a single vec
            }).flatten().collect::<Vec<_>>();

        Ok(())
    }

    pub fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device
    ) {
        self.uniforms.iter_mut().for_each(
            |uo| uo.destroy(vk_device)
        );
        // destroy the layout 
        unsafe {
            vk_device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}