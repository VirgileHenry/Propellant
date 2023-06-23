use std::fmt::Debug;

use foundry::ComponentTable;
use crate::engine::errors::PResult;
use super::uniform_buffer::{UniformBufferBuilder, UniformBuffer};

pub(crate) mod camera_uniform;
pub(crate) mod main_directionnal_light;

/// Trait for any type that can be used as a per frame uniform.
/// For this, the type needs a way to build the uniform from the component table.
pub trait AsPerFrameUniform {
    fn get_uniform(components: &ComponentTable) -> Self;
}

/// Handle around a UniformBufferBuilder<Any> used as a per frame uniform.
pub trait FrameUniformBuilder: Debug {
    fn build(
        &self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
    ) -> PResult<Box<dyn FrameUniform>>;
    fn descriptor_type(&self) -> vulkanalia::vk::DescriptorType;
}

/// This implementation basically means any T can be a uniform.
impl<T: AsPerFrameUniform + Debug + 'static> FrameUniformBuilder for UniformBufferBuilder<T> {
    fn build(
        &self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
    ) -> PResult<Box<dyn FrameUniform>> {
        Ok(Box::new(self.build(
            vk_device,
            vk_descriptor_pool,
            swapchain_images_count,
        )?))
    }

    fn descriptor_type(&self) -> vulkanalia::vk::DescriptorType {
        self.descriptor_type()
    }
}


/// handle around a per frame uniform
/// It acts as the layer between our raw uniform buffer and a more abstract uniform object.
pub trait FrameUniform: Debug {
    fn map_buffers(&mut self, vk_device: &vulkanalia::Device) -> PResult<()>;
    fn update_buffer(&mut self, components: &ComponentTable, image_index: usize) -> PResult<()>;
    fn unmap_buffers(&mut self, vk_device: &vulkanalia::Device);
    fn resize_buffer(&mut self, swapchain_image_count: usize, vk_instance: &vulkanalia::Instance, vk_device: &vulkanalia::Device, vk_physical_device: vulkanalia::vk::PhysicalDevice) -> PResult<()>;
    fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet;
    fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout;
    fn destroy(&mut self, vk_device: &vulkanalia::Device);
}

/// This implementation basically means any T can be a per frame uniform, as long as it implements Debug and AsPerTraitUniform.
impl<T: AsPerFrameUniform + Debug + 'static> FrameUniform for UniformBuffer<T> {
    fn map_buffers(&mut self, vk_device: &vulkanalia::Device) -> PResult<()> {
        self.map(vk_device)
    }

    fn update_buffer(&mut self, components: &ComponentTable, image_index: usize) -> PResult<()> {
        let uniform = T::get_uniform(components);
        self.update_buffer(0, image_index, 1, &uniform);
        Ok(())
    }

    fn unmap_buffers(&mut self, vk_device: &vulkanalia::Device) {
        self.unmap(vk_device);
    }

    fn resize_buffer(&mut self, swapchain_image_count: usize, vk_instance: &vulkanalia::Instance, vk_device: &vulkanalia::Device, vk_physical_device: vulkanalia::vk::PhysicalDevice) -> PResult<()> {
        // we have a per frame uniform : the object count is 1, we have a single object.
        self.assert_buffer_size(1, swapchain_image_count, vk_instance, vk_device, vk_physical_device)
    }

    fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet {
        self.set(image_index)
    }

    fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout()
    }

    fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.destroy_buffer(vk_device)
    }
}
