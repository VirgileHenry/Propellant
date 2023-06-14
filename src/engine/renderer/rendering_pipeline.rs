use foundry::{ComponentTable, component_iterator};
use crate::{engine::{errors::PResult, mesh::mesh_renderer::MeshRenderer}, Transform};
use self::{
    per_frame_uniforms::PerFrameUniforms,
    per_object_uniforms::PerObjectUniforms,
};

use vulkanalia::vk::DeviceV1_0;

pub struct RenderingPipeline {
    pipeline: vulkanalia::vk::Pipeline,
    layout: vulkanalia::vk::PipelineLayout,
    per_frame_uniforms: PerFrameUniforms,
    per_object_uniforms: PerObjectUniforms,
    descriptor_pool: vulkanalia::vk::DescriptorPool,
}

impl RenderingPipeline {
    pub fn new(
        pipeline: vulkanalia::vk::Pipeline,
        layout: vulkanalia::vk::PipelineLayout,
        per_frame_uniforms: PerFrameUniforms,
        per_object_uniforms: PerObjectUniforms,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
    ) -> RenderingPipeline {
        RenderingPipeline {
            pipeline,
            layout,
            per_frame_uniforms,
            per_object_uniforms,
            descriptor_pool,
        }
    }

    pub fn pipeline(&self) -> vulkanalia::vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vulkanalia::vk::PipelineLayout {
        self.layout
    }

    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        unsafe {
            vk_device.destroy_descriptor_pool(self.descriptor_pool, None);
            self.per_frame_uniforms.destroy(vk_device);
            vk_device.destroy_pipeline(self.pipeline, None);
            vk_device.destroy_pipeline_layout(self.layout, None);
        }
    }

    pub fn bind_per_frame_uniform(
        &self,
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        image_index: usize,
    ) {
        self.per_frame_uniforms.bind(
            vk_device,
            command_buffer,
            self.layout,
            image_index,
        );
    }

    pub fn bind_per_object_uniform(
        &self,
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        image_index: usize,
        swapchain_images_count: usize,
        entity: foundry::Entity,
    ) {
        self.per_object_uniforms.bind(
            vk_device,
            command_buffer,
            self.layout,
            image_index,
            swapchain_images_count,
            entity,
        );
    }

    pub fn update_uniforms(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        swapchain_images_count: usize,
        components: &mut ComponentTable,
        delta_time: f32,
    ) -> PResult<()> {
        // update every object uniform
        self.per_frame_uniforms.update(vk_device, image_index, components, delta_time)?;
        for (entity, (transform, mesh_renderer)) in component_iterator!(components; mut Transform, MeshRenderer) {
            if self.per_object_uniforms.update(vk_device, image_index, swapchain_images_count, entity, transform, mesh_renderer.material(), delta_time)? {
                // println!("scene recreation request");
            }
        }
        Ok(())
    }

    pub fn recreate_uniform_buffers(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        swapchain_images_count: usize,
        components: &ComponentTable,
    ) -> PResult<()> {
        self.per_object_uniforms.scene_recreation(vk_instance, vk_device, vk_physical_device, swapchain_images_count, components)
    }

}

pub(crate) mod camera_uniform;
pub(crate) mod model_transform_uniform;
pub(crate) mod per_frame_uniforms;
pub(crate) mod per_object_uniforms;
pub(crate) mod uniform_descriptor_set;