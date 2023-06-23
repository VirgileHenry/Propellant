use std::collections::BTreeMap;

use crate::{
    Transform,
    Material,
    engine::errors::PResult,
    ProppellantResources
};

use vulkanalia::vk::DeviceV1_0;

use self::uniform::{
    frame_uniform::FrameUniform,
    object_uniform::ObjectUniform, 
    resource_uniform::ResourceUniform
};

pub(crate) mod rendering_pipeline_builder;
pub(crate) mod uniform;
pub(crate) mod attachments;

pub struct RenderingPipeline {
    pipeline: vulkanalia::vk::Pipeline,
    layout: vulkanalia::vk::PipelineLayout,
    descriptor_pool: vulkanalia::vk::DescriptorPool,
    resource_uniforms: Vec<Box<dyn ResourceUniform>>,
    frame_uniforms: Vec<Box<dyn FrameUniform>>,
    object_uniforms: Vec<Box<dyn ObjectUniform>>,
    instance_count: usize,
    rendering_map: BTreeMap<u64, (u32, u32)>, // mesh id, (first instance, instance count)
}

impl RenderingPipeline {
    pub fn new(
        pipeline: vulkanalia::vk::Pipeline,
        layout: vulkanalia::vk::PipelineLayout,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        resource_uniforms: Vec<Box<dyn ResourceUniform>>,
        frame_uniforms: Vec<Box<dyn FrameUniform>>,
        object_uniforms: Vec<Box<dyn ObjectUniform>>,
    ) -> RenderingPipeline {
        RenderingPipeline {
            pipeline,
            layout,
            descriptor_pool,
            resource_uniforms,
            frame_uniforms,
            object_uniforms,
            instance_count: 0,
            rendering_map: BTreeMap::new(),
        }
    }

    pub fn pipeline(&self) -> vulkanalia::vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vulkanalia::vk::PipelineLayout {
        self.layout
    }

    pub fn register_draw_commands(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        command_buffer: vulkanalia::vk::CommandBuffer,
        resources: &ProppellantResources,
    ) {
        // bind the pipeline 
        unsafe {
            vk_device.cmd_bind_pipeline(
                command_buffer,
                vulkanalia::vk::PipelineBindPoint::GRAPHICS,
                self.pipeline
            );
        }

        // bind all descriptor sets
        let empty_ds = Vec::with_capacity(0);
        let ds = empty_ds.into_iter()
            .chain(self.resource_uniforms.iter_mut().map(|uniform| uniform.set(image_index)))
            .chain(self.frame_uniforms.iter().map(|uniform| uniform.set(image_index)))
            .chain(self.object_uniforms.iter().map(|uniform| uniform.set(image_index)))
            .collect::<Vec<_>>();

        unsafe {
            vk_device.cmd_bind_descriptor_sets(
                command_buffer,
                vulkanalia::vk::PipelineBindPoint::GRAPHICS,
                self.layout,
                0,
                &ds,
                &[]
            );
        }
        
        // for each concerned mesh; bind it and draw instanced !
        for (mesh_id, (first_instance, instance_count)) in self.rendering_map.iter() {
            match resources.meshes().loaded_mesh(mesh_id) {
                Some(loaded_mesh) => {
                    // bind the mesh vertex and index
                    loaded_mesh.bind_mesh(vk_device, command_buffer);
                    unsafe {
                        vk_device.cmd_draw_indexed(
                            command_buffer,
                            loaded_mesh.index_count() as u32,
                            *instance_count,
                            0,
                            0,
                            *first_instance as u32
                        );
                    }
                },
                None => {
                    if cfg!(debug_assertions) {
                        println!("[PROPELLANT DEBUG] Mesh not in mesh library (id {})", mesh_id);
                    }
                }
            }
        }
    }

    pub fn map_all_uniform_buffers(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
    ) -> PResult<()> {
        // check if we have at least one entity to draw
        if self.rendering_map.is_empty() {
            return Ok(());
        }

        for frame_uniform in self.frame_uniforms.iter_mut() {
            frame_uniform.map_buffers(vk_device, image_index)?;
        }
        for object_uniform in self.object_uniforms.iter_mut() {
            object_uniform.map_buffers(vk_device, image_index)?;
        }

        Ok(())
    }

    pub fn update_frame_uniform_buffers(
        &mut self,
        components: &foundry::ComponentTable,
        image_index: usize,
    ) -> PResult<()> {
        // check if we have at least one entity to draw
        if self.rendering_map.is_empty() {
            return Ok(());
        }

        for frame_uniform in self.frame_uniforms.iter_mut() {
            frame_uniform.update_buffer(components, image_index)?;
        }

        Ok(())
    }

    pub fn update_uniform_buffers(
        &mut self,
        instance_id: usize,
        transform: &Transform,
        material: &Material,
        image_index: usize,
    ) -> PResult<()> {
        // check if we have at least one entity to draw
        if self.rendering_map.is_empty() {
            return Ok(());
        }

        for object_uniform in self.object_uniforms.iter_mut() {
            object_uniform.update_buffer(instance_id, transform, material, image_index)?;
        }

        Ok(())
    }

    pub fn unmap_all_uniform_buffers(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
    ) {
        // check if we have at least one entity to draw
        if self.rendering_map.is_empty() {
            return;
        }

        for frame_uniform in self.frame_uniforms.iter_mut() {
            frame_uniform.unmap_buffers(vk_device, image_index);
        }
        for object_uniform in self.object_uniforms.iter_mut() {
            object_uniform.unmap_buffers(vk_device, image_index);
        }
    }

    /// recreate the scene: objects were created or destroyed.
    /// The mesh map is a mapping of mesh id to (object_count, instance_offset, object count).
    /// The doubling of the first and 3rd numbe comes because it have been used already to count offsets, do not care.
    pub fn resize_uniforms_buffers(
        &mut self,
        mesh_map: BTreeMap<u64, (usize, usize, usize)>,
        image_index: usize,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
    ) -> PResult<()> {
        // the total object count can be easily computed from the mesh map
        self.instance_count = mesh_map.iter().map(|(_, v)| v.0).sum();

        for frame_uniform in self.frame_uniforms.iter_mut() {
            frame_uniform.resize_buffer(image_index, vk_instance, vk_device, vk_physical_device)?;
        }

        for object_uniform in self.object_uniforms.iter_mut() {
            object_uniform.resize_buffer(self.instance_count, image_index, vk_instance, vk_device, vk_physical_device)?;
        }

        // finally, consume the btree map to recreate our rendering map
        self.rendering_map = mesh_map.into_iter().map(|(k, v)| (k, (v.1 as u32, v.0 as u32))).collect();

        Ok(())
    }

    /// Reload the resource uniforms.
    /// This should be called when a resource is reloaded.
    pub fn rebuild_resources_uniforms(
        &mut self,
        vk_device: &vulkanalia::Device,
        resources: &ProppellantResources,
    ) -> PResult<()> {
        for uniform in self.resource_uniforms.iter_mut() {
            uniform.recreate(vk_device, self.descriptor_pool, resources)?;
        }

        Ok(())
    }

    pub fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        // clean up uniforms
        for resource_uniform in self.resource_uniforms.iter_mut() {
            resource_uniform.destroy(vk_device);
        }
        for frame_uniform in self.frame_uniforms.iter_mut() {
            frame_uniform.destroy(vk_device);
        }
        for object_uniform in self.object_uniforms.iter_mut() {
            object_uniform.destroy(vk_device);
        }
        unsafe {
            vk_device.destroy_descriptor_pool(self.descriptor_pool, None);
            vk_device.destroy_pipeline(self.pipeline, None);
            vk_device.destroy_pipeline_layout(self.layout, None);
        }
    }

}
