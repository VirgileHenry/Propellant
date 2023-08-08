use std::fmt::Debug;

use crate::engine::errors::PResult;

pub(crate) mod textures_uniform;


pub trait ResourceUniformBuilder: Debug {
    fn build(
        &self,
        vk_device: &vulkanalia::Device,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
    ) -> PResult<Box<dyn ResourceUniform>>;

    fn descriptor_type(&self) -> vulkanalia::vk::DescriptorType;
}


pub trait ResourceUniform : Debug {
    fn recreate(&mut self, vk_device: &vulkanalia::Device, descriptor_pool: vulkanalia::vk::DescriptorPool, resources: &crate::ProppellantResources) -> PResult<()>;
    fn set(&self, image_index: usize) -> vulkanalia::vk::DescriptorSet;
    fn layout(&self) -> vulkanalia::vk::DescriptorSetLayout;
    fn destroy(&mut self, vk_device: &vulkanalia::Device);
}