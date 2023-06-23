use std::fmt::Debug;

use crate::{engine::errors::PResult, Transform, Material};
use super::uniform_buffer::{UniformBufferBuilder, UniformBuffer};

pub(crate) mod model_uniform;

/// Trait for any type that can be used as a per frame uniform.
/// For this, the type needs a way to build the uniform from the component table.
pub trait AsPerObjectUniform {
    fn get_uniform(transform: &Transform, material: &Material) -> PResult<Self> where Self: Sized;
}


pub trait ObjectUniformBuilder: Debug {
    fn build(
        &self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
    ) -> PResult<Box<dyn ObjectUniform>>;
    fn descriptor_type(&self) -> vulkanalia::vk::DescriptorType;
}

impl<T: AsPerObjectUniform + Debug + 'static> ObjectUniformBuilder for UniformBufferBuilder<T> {
    fn build(
        &self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        swapchain_images_count: usize,
    ) -> PResult<Box<dyn ObjectUniform>> {
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


/// A handle around a UniformBuffer<Any> used as a per object uniform.
/// It acts as the layer between our raw uniform buffer and a more abstract uniform object.
pub trait ObjectUniform: Debug {
    fn map_buffers(&mut self, vk_device: &vulkanalia::Device) -> PResult<()>;
    fn update_buffer(&mut self, instance_id: usize, instance_count: usize, transform: &Transform, material: &Material, image_index: usize) -> PResult<()>;
    fn unmap_buffers(&mut self, vk_device: &vulkanalia::Device);
    fn resize_buffer(&mut self, object_count: usize, swapchain_image_count: usize, vk_instance: &vulkanalia::Instance, vk_device: &vulkanalia::Device, vk_physical_device: vulkanalia::vk::PhysicalDevice) -> PResult<()>;
    fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet;
    fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout;
    fn destroy(&mut self, vk_device: &vulkanalia::Device);
}

/// This implementation basically means any T can be an object uniform, as long as it implements the Debug and AsPerObjectUniform traits.
impl<T: AsPerObjectUniform + Debug + 'static> ObjectUniform for UniformBuffer<T> {
    fn map_buffers(&mut self, vk_device: &vulkanalia::Device) -> PResult<()> {
        self.map(vk_device)
    }

    fn update_buffer(&mut self, instance_id: usize, instance_count: usize, transform: &Transform, material: &Material, image_index: usize) -> PResult<()> {
        let uniform = T::get_uniform(transform, material)?;
        self.update_buffer(instance_id, image_index, instance_count, &uniform);
        Ok(())
    }

    fn unmap_buffers(&mut self, vk_device: &vulkanalia::Device) {
        self.unmap(vk_device);
    }

    fn resize_buffer(&mut self, object_count: usize, swapchain_image_count: usize, vk_instance: &vulkanalia::Instance, vk_device: &vulkanalia::Device, vk_physical_device: vulkanalia::vk::PhysicalDevice) -> PResult<()> {
        self.assert_buffer_size(object_count, swapchain_image_count, vk_instance, vk_device, vk_physical_device)
    }

    fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet {
        self.set(image_index)
    }

    fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout {
        self.layout()
    }

    fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.destroy_buffer(vk_device);
    }
}