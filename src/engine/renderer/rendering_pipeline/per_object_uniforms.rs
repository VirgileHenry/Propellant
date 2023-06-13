use std::collections::HashMap;

use foundry::{Entity, component_iterator, ComponentTable};
use crate::{Transform, engine::{mesh::mesh_renderer::MeshRenderer, errors::PResult}, Material};
use super::uniform_descriptor_set::per_object_uniform::PerObjectUniformObject;

use vulkanalia::vk::DeviceV1_0;

#[allow(unused)]
const PER_OBJECT_SET_BINDING: u32 = 2;

pub struct PerObjectUniforms {
    /// The per object uniforms.
    uniforms: Vec<PerObjectUniformObject>,
    /// A map that for each entity, gives the position in the buffer.
    entity_to_index: HashMap<Entity, usize>,
    /// Number of entities to render.
    entity_count: usize,
}

impl PerObjectUniforms {
    pub fn new(uniforms: Vec<PerObjectUniformObject>) -> PerObjectUniforms {
        PerObjectUniforms {
            uniforms,
            entity_to_index: HashMap::new(),
            entity_count: 0,
        }
    }

    pub fn bind(
        &self,
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        layout: vulkanalia::vk::PipelineLayout,
        image_index: usize,
        swapchain_images_count: usize,
        entity: Entity,
    ) -> bool {
        // get the buffer offset for the entity
        let offset = match self.entity_to_index.get(&entity) {
            Some(offset) => *offset,
            None => return true,
        };
        // create an array of the descriptor sets
        let descriptor_sets = self.uniforms.iter()
            .map(|uo| uo.set(image_index))
            .collect::<Vec<_>>();
        // create an array of the offsets
        let offsets = self.uniforms.iter()
            .map(|uo| uo.byte_offset(offset, image_index, swapchain_images_count) as u32)
            .collect::<Vec<_>>();

        // bind all the descriptor sets
        unsafe {
            vk_device.cmd_bind_descriptor_sets(
                command_buffer,
                vulkanalia::vk::PipelineBindPoint::GRAPHICS,
                layout,
                1,
                &descriptor_sets,
                &offsets,
            );
        }

        false
    }

    /// Update the uniforms buffers for the objects.
    /// If the entity is not in the entity to index map, things have been moving : we need scene recreation.
    pub fn update(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        swapchain_images_count: usize,
        entity: Entity,
        transform: &Transform,
        material: &Material,
        delta_time: f32,
    ) -> PResult<bool> {
        // get the entity offset
        let offset = match self.entity_to_index.get(&entity) {
            Some(offset) => *offset,
            None => return Ok(true),
        };
        // per frame uniforms
        for uniform_object in self.uniforms.iter_mut() {
            uniform_object.update_buffer(vk_device, image_index, swapchain_images_count, transform, material, offset, delta_time)?;
        }

        Ok(false)
    }

    /// recompute the uniforms buffers for the objects.
    pub fn scene_recreation(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        swapchain_images_count: usize,
        components: &ComponentTable,
    ) -> PResult<()> {
        self.entity_count = 0;
        // create the entity to offset map
        self.entity_to_index.clear();
        for (offset, (entity, _)) in component_iterator!(components; mut Transform, MeshRenderer).enumerate() {
            self.entity_to_index.insert(entity, offset as usize);
            self.entity_count += 1;
        }

        // tell every uniform to rebuild their buffers.
        for uniform in self.uniforms.iter_mut() {
            uniform.resize_buffer(
                vk_instance,
                vk_device,
                vk_physical_device,
                swapchain_images_count,
                self.entity_count,
            )?;
        }
        // now, we need te reset any uniforms that do not update every frame.
        // we do the same iteration, allow us to skip asking for offsets to the hashmap.
        for (offset, (_, (transform, mesh_renderer))) in component_iterator!(components; mut Transform, MeshRenderer).enumerate() {
            for uniform in self.uniforms.iter_mut() {
                uniform.update_start_only_buffer(
                    vk_device,
                    swapchain_images_count,
                    transform,
                    mesh_renderer.material(),
                    offset
                )?;
            }
        }

        Ok(())

    }
}