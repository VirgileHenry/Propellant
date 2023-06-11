use foundry::ComponentTable;
use crate::engine::errors::PropellantError;
use self::uniform_descriptor_set::per_frame_uniform::PerFrameUniformObject;

use vulkanalia::vk::DeviceV1_0;

pub struct RenderingPipeline {
    pipeline: vulkanalia::vk::Pipeline,
    layout: vulkanalia::vk::PipelineLayout,
    per_frame_uniforms: Vec<PerFrameUniformObject>, 
    descriptor_pool: vulkanalia::vk::DescriptorPool,
}

impl RenderingPipeline {
    pub fn new(
        pipeline: vulkanalia::vk::Pipeline,
        layout: vulkanalia::vk::PipelineLayout,
        per_frame_uniforms: Vec<PerFrameUniformObject>, 
        descriptor_pool: vulkanalia::vk::DescriptorPool,
    ) -> RenderingPipeline {
        RenderingPipeline {
            pipeline,
            layout,
            per_frame_uniforms,
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
            self.per_frame_uniforms.iter_mut().for_each(|uo| uo.destroy(vk_device));
            vk_device.destroy_pipeline(self.pipeline, None);
            vk_device.destroy_pipeline_layout(self.layout, None);
        }
    }

    pub fn bind_descriptor_sets(
        &self,
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        image_index: usize,
    ) {
        // only per frame for now
        // create a vec referecing all per frame descriptor sets
        let descriptor_sets = self.per_frame_uniforms.iter()
            .map(|uo| uo.set(image_index))
            .collect::<Vec<_>>();
        // bind all the descriptor sets
        unsafe {
            vk_device.cmd_bind_descriptor_sets(
                command_buffer,
                vulkanalia::vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                &descriptor_sets,
                &[],
            );
        }

        // todo; per object
    }

    pub fn set_uniforms(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        components: &mut ComponentTable,
    ) -> Result<(), PropellantError> {
        // per frame uniforms
        for uniform_object in self.per_frame_uniforms.iter_mut() {
            uniform_object.update_buffer(vk_device, image_index, components)?;
        }

        Ok(())
    }

}

pub(crate) mod uniform_descriptor_set;
pub(crate) mod camera_uniform;