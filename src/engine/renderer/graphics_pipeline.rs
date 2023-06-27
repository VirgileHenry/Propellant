use std::collections::BTreeMap;
use std::collections::HashMap;

use crate::{
    Transform,
    Material,
    engine::errors::PResult,
    ProppellantResources
};

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::Handle;

use self::graphic_pipeline_state::GraphicPipelineCreationState;
use self::uniform::{
    frame_uniform::FrameUniform,
    object_uniform::ObjectUniform, 
    resource_uniform::ResourceUniform
};

pub(crate) mod attachments;
pub(crate) mod graphics_pipeline_builder;
pub(crate) mod graphic_pipeline_state;
pub(crate) mod uniform;

pub struct GraphicsPipeline {
    pipeline: vulkanalia::vk::Pipeline,
    pipeline_layout: vulkanalia::vk::PipelineLayout,
    descriptor_pool: vulkanalia::vk::DescriptorPool,
    creation_state: GraphicPipelineCreationState,
    resource_uniforms: Vec<Box<dyn ResourceUniform>>,
    frame_uniforms: Vec<Box<dyn FrameUniform>>,
    object_uniforms: Vec<Box<dyn ObjectUniform>>,
    instance_count: usize,
    rendering_map: BTreeMap<u64, (u32, u32)>, // mesh id, (first instance, instance count)
}

impl GraphicsPipeline {
    pub fn create(
        vk_device: &vulkanalia::Device,
        vertex_binding_description: Vec<vulkanalia::vk::VertexInputBindingDescription>,
        vertex_attribute_description: Vec<vulkanalia::vk::VertexInputAttributeDescription>,
        shader_stages: HashMap<vulkanalia::vk::ShaderStageFlags, vulkanalia::vk::ShaderModule>,
        swapchain_extent: vulkanalia::vk::Extent2D,
        pipeline_layout: vulkanalia::vk::PipelineLayout,
        render_pass: vulkanalia::vk::RenderPass,
        descriptor_pool: vulkanalia::vk::DescriptorPool,
        resource_uniforms: Vec<Box<dyn ResourceUniform>>,
        frame_uniforms: Vec<Box<dyn FrameUniform>>,
        object_uniforms: Vec<Box<dyn ObjectUniform>>,
    ) -> PResult<GraphicsPipeline> {

        let vertex_input_state = vulkanalia::vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_binding_description)
            .vertex_attribute_descriptions(&vertex_attribute_description)
            .build();
        
        // here, default values to draw triangles. Maybe to rework at some point ? 
        let input_assembly_state = vulkanalia::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vulkanalia::vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();
        
        
        // create a default sized viewport.
        // here, we could draw to only part of the screen for local multiplayer.
        let viewport = vulkanalia::vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        
        // scissors are like masks, here use the entire screen to draw everything
        let scissor = vulkanalia::vk::Rect2D::builder()
            .offset(vulkanalia::vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_extent);       
        
        // they are put into arrays, as they could be mutliple of them, but this require a specific extension.
        let viewports = &[viewport];
        let scissors = &[scissor];
        let viewport_state = vulkanalia::vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors);
        
        // create the rasterizer
        let rasterization_state = vulkanalia::vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vulkanalia::vk::PolygonMode::FILL) // can be set to fill here !
            .line_width(1.0)
            .cull_mode(vulkanalia::vk::CullModeFlags::BACK)
            .front_face(vulkanalia::vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false)
            .build();
        
        // multisampling state: antialiasing here
        let multisample_state = vulkanalia::vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vulkanalia::vk::SampleCountFlags::_1)
            .build();
        
        // color blending. transparency and alpha color blending can be done here !
        let color_attachment = vulkanalia::vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vulkanalia::vk::ColorComponentFlags::all())
            .blend_enable(false)
            .build();

        // todo : depth attachment 

        let color_blend_attachments = vec![color_attachment];
        let color_blend_state = vulkanalia::vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vulkanalia::vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();        

        // create the pipeline ! 
        let stages = shader_stages.iter().map(|(stage, shader_module)|
            vulkanalia::vk::PipelineShaderStageCreateInfo::builder()
                .stage(*stage)
                .module(*shader_module)
                .name(b"main\0")
                .build()
        ).collect::<Vec<_>>();

        let creation_state = GraphicPipelineCreationState {
            stages,
            vertex_input_state,
            vertex_binding_description,
            vertex_attribute_description,
            input_assembly_state,
            rasterization_state,
            multisample_state,
            color_blend_state,
            color_blend_attachments,
        };

        let info = vulkanalia::vk::GraphicsPipelineCreateInfo::builder()
            .stages(&creation_state.stages)
            .vertex_input_state(&creation_state.vertex_input_state)
            .input_assembly_state(&creation_state.input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&creation_state.rasterization_state)
            .multisample_state(&creation_state.multisample_state)
            .color_blend_state(&creation_state.color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_handle(vulkanalia::vk::Pipeline::null()) // Optional.
            .base_pipeline_index(-1); // Optional.

        let pipeline = unsafe {
            vk_device.create_graphics_pipelines(vulkanalia::vk::PipelineCache::null(), &[info], None)?.0
        };

        Ok(GraphicsPipeline {
            pipeline,
            pipeline_layout,
            descriptor_pool,
            creation_state,
            resource_uniforms,
            frame_uniforms,
            object_uniforms,
            instance_count: 0,
            rendering_map: BTreeMap::new(),
        })
    }

    pub fn recreation_cleanup(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        unsafe {
            vk_device.destroy_pipeline(self.pipeline, None);
        }
    }

    pub fn recreate(
        &mut self,
        vk_device: &vulkanalia::Device,
        swapchain_extent: vulkanalia::vk::Extent2D,
        render_pass: vulkanalia::vk::RenderPass,
    ) -> PResult<()> {
        // create a default sized viewport.
        let viewport = vulkanalia::vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain_extent.width as f32)
            .height(swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);
        
        // scissors are like masks, here use the entire screen to draw everything
        let scissor = vulkanalia::vk::Rect2D::builder()
            .offset(vulkanalia::vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_extent);       
        
        // they are put into arrays, as they could be mutliple of them, but this require a specific extension.
        let viewports = &[viewport];
        let scissors = &[scissor];
        let viewport_state = vulkanalia::vk::PipelineViewportStateCreateInfo::builder()
            .viewports(viewports)
            .scissors(scissors);
        
        let info = vulkanalia::vk::GraphicsPipelineCreateInfo::builder()
            .stages(&self.creation_state.stages)
            .vertex_input_state(&self.creation_state.vertex_input_state)
            .input_assembly_state(&self.creation_state.input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&self.creation_state.rasterization_state)
            .multisample_state(&self.creation_state.multisample_state)
            .color_blend_state(&self.creation_state.color_blend_state)
            .layout(self.pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_handle(vulkanalia::vk::Pipeline::null()) // Optional.
            .base_pipeline_index(-1); // Optional.

        self.pipeline = unsafe {
            vk_device.create_graphics_pipelines(vulkanalia::vk::PipelineCache::null(), &[info], None)?.0
        };

        Ok(())
    }

    pub fn pipeline(&self) -> vulkanalia::vk::Pipeline {
        self.pipeline
    }

    pub fn layout(&self) -> vulkanalia::vk::PipelineLayout {
        self.pipeline_layout
    }

    pub fn register_draw_commands(
        &self,
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
            .chain(self.resource_uniforms.iter().map(|uniform| uniform.set(image_index)))
            .chain(self.frame_uniforms.iter().map(|uniform| uniform.set(image_index)))
            .chain(self.object_uniforms.iter().map(|uniform| uniform.set(image_index)))
            .collect::<Vec<_>>();

        unsafe {
            vk_device.cmd_bind_descriptor_sets(
                command_buffer,
                vulkanalia::vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
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
        self.creation_state.destroy(vk_device);
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
            vk_device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }

}
