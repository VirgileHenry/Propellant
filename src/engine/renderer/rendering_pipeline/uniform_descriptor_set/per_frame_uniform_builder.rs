use std::rc::Rc;

use foundry::ComponentTable;
use crate::engine::{window::vulkan::vulkan_buffer::VulkanBuffer, errors::PResult};
use super::uniform_update_frequency::UniformUpdateFrequency;

pub struct PerFrameUniformBuilder {
    buffer_update: Rc<dyn Fn(
        &vulkanalia::Device,
        usize,
        &ComponentTable,
        &mut VulkanBuffer,
    ) -> PResult<()>>,
    object_size: usize,
    stage: vulkanalia::vk::ShaderStageFlags,
    binding: u32,
    update_frequency: UniformUpdateFrequency,
}

impl std::fmt::Debug for PerFrameUniformBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerFrameUniformBuilder")
            .finish()
    }
}

impl PerFrameUniformBuilder {
    pub fn new<T: 'static>(
        object_generator: fn(&ComponentTable) -> PResult<T>,
        stage: vulkanalia::vk::ShaderStageFlags,
        binding: u32,
        update_frequency: UniformUpdateFrequency,
    ) -> PerFrameUniformBuilder {
        PerFrameUniformBuilder {
            buffer_update: Rc::new(move |
                vk_device: &vulkanalia::Device,
                image_index: usize,
                components: &ComponentTable,
                buffer: &mut VulkanBuffer,
            | {
                object_generator(components).and_then(
                    |object| buffer.map_data(vk_device, &[object], image_index * std::mem::size_of::<T>())
                )
            }),
            object_size: std::mem::size_of::<T>(),
            stage,
            binding,
            update_frequency,
        }
    }

    pub fn buffer_update(&self) -> Rc<dyn Fn(
        &vulkanalia::Device,
        usize,
        &ComponentTable,
        &mut VulkanBuffer,
    ) -> PResult<()>> {
        self.buffer_update.clone()
    }

    pub fn object_size(&self) -> usize {
        self.object_size
    }

    pub fn stage(&self) -> vulkanalia::vk::ShaderStageFlags {
        self.stage
    }

    pub fn binding(&self) -> u32 {
        self.binding
    }

    pub fn update_frequency(&self) -> &UniformUpdateFrequency {
        &self.update_frequency
    }
}