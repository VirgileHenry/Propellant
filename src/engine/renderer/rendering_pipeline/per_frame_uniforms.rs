use foundry::ComponentTable;
use crate::engine::errors::PResult;
use super::uniform_descriptor_set::per_frame_uniform::PerFrameUniformObject;

use vulkanalia::vk::DeviceV1_0;



pub struct PerFrameUniforms {
    uniforms: Vec<PerFrameUniformObject>,
}

impl PerFrameUniforms {
    pub fn new(uniforms: Vec<PerFrameUniformObject>) -> PerFrameUniforms {
        PerFrameUniforms {
            uniforms,
        }
    }

    pub fn bind(
        &self,
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        layout: vulkanalia::vk::PipelineLayout,
        image_index: usize,
    ) {
        // only per frame for now
        // create a vec referecing all per frame descriptor sets
        let descriptor_sets = self.uniforms.iter()
            .map(|uo| uo.set(image_index))
            .collect::<Vec<_>>();
        // bind all the descriptor sets
        unsafe {
            vk_device.cmd_bind_descriptor_sets(
                command_buffer,
                vulkanalia::vk::PipelineBindPoint::GRAPHICS,
                layout,
                0,
                &descriptor_sets,
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


    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device
    ) {
        self.uniforms.iter_mut().for_each(
            |uo| uo.destroy(vk_device)
        );
    }
}