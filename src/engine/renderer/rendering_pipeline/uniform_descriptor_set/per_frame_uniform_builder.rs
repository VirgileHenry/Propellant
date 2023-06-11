use std::rc::Rc;

use foundry::ComponentTable;

use crate::engine::{window::vulkan::vulkan_buffer::VulkanBuffer, errors::PropellantError};

pub struct PerFrameUniformBuilder {
    object_getter: Rc<dyn Fn(
        &vulkanalia::Device,
        usize,
        &mut ComponentTable,
        &mut VulkanBuffer,
    ) -> Result<(), PropellantError>>,
    object_size: usize,
}

impl std::fmt::Debug for PerFrameUniformBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerFrameUniformBuilder")
            .finish()
    }
}

impl PerFrameUniformBuilder {
    pub fn new<T: 'static>(generator: fn(&ComponentTable) -> Result<T, PropellantError>) -> PerFrameUniformBuilder {
        PerFrameUniformBuilder {
            object_getter: Rc::new(move |
                vk_device: &vulkanalia::Device,
                image_index: usize,
                components: &mut ComponentTable,
                buffer: &mut VulkanBuffer,
            | {
                generator(components).and_then(
                    |object| buffer.map_data(vk_device, &[object], image_index * std::mem::size_of::<T>())
                )
            }),
            object_size: std::mem::size_of::<T>(),
        }
    }

    pub fn builder(&self) -> Rc<dyn Fn(
        &vulkanalia::Device,
        usize,
        &mut ComponentTable,
        &mut VulkanBuffer,
    ) -> Result<(), PropellantError>> {
        self.object_getter.clone()
    }

    pub fn object_size(&self) -> usize {
        self.object_size
    }
}