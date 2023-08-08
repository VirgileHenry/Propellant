use std::collections::HashMap;

use crate::engine::consts::PROPELLANT_DEBUG_FEATURES;
use crate::{
    engine::errors::PResult,
    ProppellantResources
};

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::Handle;

use self::graphic_pipeline_state::GraphicPipelineCreationState;
use self::renderable_component::RenderableComponent;
use self::uniform::uniform_buffer::UniformBuffer;
use self::uniform::{
    frame_uniform::FrameUniform,
    object_uniform::ObjectUniform, 
};

use super::rendering_map::RenderingMap;

pub(crate) mod graphic_pipeline_builder;
pub(crate) mod graphic_pipeline_state;
pub(crate) mod renderable_component;
pub(crate) mod uniform;

/// This trait is a handle around graphic pipelines.
/// Graphic pipelines are generic over the uniforms, so they can't be stored directly.
/// Instead, we hide them behind this trait, that provide access to all it's method.
pub trait GraphicPipelineInterface {
    fn recreation_cleanup(
        &mut self,
        vk_device: &vulkanalia::Device,
    );
    fn recreate(
        &mut self,
        vk_device: &vulkanalia::Device,
        swapchain_extent: vulkanalia::vk::Extent2D,
        render_pass: vulkanalia::vk::RenderPass,
    ) -> PResult<()>;
    fn register_draw_commands(
        &self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        command_buffer: vulkanalia::vk::CommandBuffer,
        resources: &ProppellantResources,
    );
    fn update_uniform_buffers(
        &mut self,
        vk_device: &vulkanalia::Device,
        components: &foundry::ComponentTable,
        image_index: usize,
    ) -> PResult<()>;
    fn rebuild_rendering_map(
        &mut self,
        components: &foundry::ComponentTable,
    );
    fn assert_uniform_buffer_sizes(
        &mut self,
        image_index: usize,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
    ) -> PResult<()>;
    fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    );
}


pub struct GraphicPipelineFFFOO<
    FrameUniform1: FrameUniform,
    FrameUniform2: FrameUniform,
    FrameUniform3: FrameUniform,
    ObjectUniform1: RenderableComponent,
    ObjectUniform2: ObjectUniform
> {
    phantom: std::marker::PhantomData<(FrameUniform1, FrameUniform2, FrameUniform3, ObjectUniform1, ObjectUniform2)>,
    frame_1_uniform_buffer: UniformBuffer<FrameUniform1>,
    frame_2_uniform_buffer: UniformBuffer<FrameUniform2>,
    frame_3_uniform_buffer: UniformBuffer<FrameUniform3>,
    object_1_uniform_buffer: UniformBuffer<ObjectUniform1>,
    object_2_uniform_buffer: UniformBuffer<ObjectUniform2>,
    pipeline: vulkanalia::vk::Pipeline,
    pipeline_layout: vulkanalia::vk::PipelineLayout,
    vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
    creation_state: GraphicPipelineCreationState,
    rendering_map: RenderingMap,
}

impl<
    FrameUniform1: FrameUniform + 'static,
    FrameUniform2: FrameUniform + 'static,
    FrameUniform3: FrameUniform + 'static,
    ObjectUniform1: RenderableComponent + 'static,
    ObjectUniform2: ObjectUniform + 'static,
> GraphicPipelineFFFOO<FrameUniform1, FrameUniform2, FrameUniform3, ObjectUniform1, ObjectUniform2> {
    fn create(
        vk_device: &vulkanalia::Device,
        vertex_binding_description: Vec<vulkanalia::vk::VertexInputBindingDescription>,
        vertex_attribute_description: Vec<vulkanalia::vk::VertexInputAttributeDescription>,
        shader_stages: HashMap<vulkanalia::vk::ShaderStageFlags, vulkanalia::vk::ShaderModule>,
        swapchain_extent: vulkanalia::vk::Extent2D,
        pipeline_layout: vulkanalia::vk::PipelineLayout,
        render_pass: vulkanalia::vk::RenderPass,
        vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
        frame_1_uniform_buffer: UniformBuffer<FrameUniform1>,
        frame_2_uniform_buffer: UniformBuffer<FrameUniform2>,
        frame_3_uniform_buffer: UniformBuffer<FrameUniform3>,
        object_1_uniform_buffer: UniformBuffer<ObjectUniform1>,
        object_2_uniform_buffer: UniformBuffer<ObjectUniform2>,
    ) -> PResult<GraphicPipelineFFFOO<FrameUniform1, FrameUniform2, FrameUniform3, ObjectUniform1, ObjectUniform2>> {

        let vertex_input_state = vulkanalia::vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_binding_description)
            .vertex_attribute_descriptions(&vertex_attribute_description)
            .build();
        
        // here, default values to draw triangles. Maybe to rework at some point ? 
        let input_assembly_state = vulkanalia::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vulkanalia::vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();
        
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

        let color_blend_attachments = vec![color_attachment];

        let color_blend_state = vulkanalia::vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vulkanalia::vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        let depth_stencil_state = vulkanalia::vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vulkanalia::vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
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
            depth_stencil_state,
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
            .depth_stencil_state(&depth_stencil_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_handle(vulkanalia::vk::Pipeline::null()) // Optional.
            .base_pipeline_index(-1); // Optional.

        let pipeline = unsafe {
            vk_device.create_graphics_pipelines(vulkanalia::vk::PipelineCache::null(), &[info], None)?.0
        };

        // todo : builders for uniforms / to know the layouts and all

        Ok(GraphicPipelineFFFOO {
            phantom: Default::default(),
            frame_1_uniform_buffer,
            frame_2_uniform_buffer,
            frame_3_uniform_buffer,
            object_1_uniform_buffer,
            object_2_uniform_buffer,
            pipeline,
            pipeline_layout,
            vk_descriptor_pool,
            creation_state,
            rendering_map: RenderingMap::new(),
        })
    }
}

impl<
    FrameUniform1: FrameUniform + 'static,
    FrameUniform2: FrameUniform + 'static,
    FrameUniform3: FrameUniform + 'static,
    ObjectUniform1: RenderableComponent + 'static,
    ObjectUniform2: ObjectUniform + 'static,
> GraphicPipelineInterface for GraphicPipelineFFFOO<FrameUniform1, FrameUniform2, FrameUniform3, ObjectUniform1, ObjectUniform2>  {
    fn recreation_cleanup(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        unsafe {
            vk_device.destroy_pipeline(self.pipeline, None);
        }
    }

    fn recreate(
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
            .depth_stencil_state(&self.creation_state.depth_stencil_state)
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

    fn update_uniform_buffers(
        &mut self,
        vk_device: &vulkanalia::Device,
        components: &foundry::ComponentTable,
        image_index: usize,
    ) -> PResult<()> {
        // map all the buffers
        self.frame_1_uniform_buffer.map(vk_device, image_index)?;
        self.frame_2_uniform_buffer.map(vk_device, image_index)?;
        self.frame_3_uniform_buffer.map(vk_device, image_index)?;
        self.object_1_uniform_buffer.map(vk_device, image_index)?;
        self.object_2_uniform_buffer.map(vk_device, image_index)?;
        // frame uniforms
        self.frame_1_uniform_buffer.update_buffer(0, image_index, FrameUniform1::get_uniform(components));
        self.frame_2_uniform_buffer.update_buffer(0, image_index, FrameUniform2::get_uniform(components));
        self.frame_3_uniform_buffer.update_buffer(0, image_index, FrameUniform3::get_uniform(components));
        // object uniforms
        for (entity_id, obj_uniform1, obj_uniform2) in components.query2d::<ObjectUniform1::FromComponent, ObjectUniform2::FromComponent>() {
            let mesh_id = ObjectUniform1::mesh_id(obj_uniform1);
            match self.rendering_map.get_buffer_index(mesh_id, entity_id) {
                Some(buffer_index) => {
                    self.object_1_uniform_buffer.update_buffer(buffer_index, image_index, ObjectUniform1::get_uniform(obj_uniform1));
                    self.object_2_uniform_buffer.update_buffer(buffer_index, image_index, ObjectUniform2::get_uniform(obj_uniform2));
                },
                None => if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] Mesh / Entity not in rendering map (Mesh {}), (Entity {}). Scene have not been built properly.", mesh_id, entity_id);
                }
            };
        }
        // unmap all the buffers
        self.frame_1_uniform_buffer.unmap(vk_device, image_index);
        self.frame_2_uniform_buffer.unmap(vk_device, image_index);
        self.frame_3_uniform_buffer.unmap(vk_device, image_index);
        self.object_1_uniform_buffer.unmap(vk_device, image_index);
        self.object_2_uniform_buffer.unmap(vk_device, image_index);

        Ok(())
    }

    fn register_draw_commands(
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
        let ds = vec![
            self.frame_1_uniform_buffer.set(image_index),
            self.frame_2_uniform_buffer.set(image_index),
            self.frame_3_uniform_buffer.set(image_index),
            self.object_1_uniform_buffer.set(image_index),
            self.object_2_uniform_buffer.set(image_index),
        ];

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
        let mut first_instance = 0;
        for (mesh, instance_count) in self.rendering_map.iter(resources) {
            mesh.bind_mesh(vk_device, command_buffer);
            unsafe {
                vk_device.cmd_draw_indexed(
                    command_buffer,
                    mesh.index_count() as u32,
                    instance_count,
                    0,
                    0,
                    first_instance
                );
            }
            first_instance += instance_count;
        }
    }

    fn rebuild_rendering_map(
        &mut self,
        components: &foundry::ComponentTable,
    ) {
        // assert buffer sizes
        // self.frame_1_uniform_buffer.assert_buffer_size(object_count, image_index, vk_instance, vk_device, vk_physical_device)
        // clear the map
        self.rendering_map.clear();
        // iterate over objects, count how many for each mesh
        // O(n) complexity
        for (entity_id, obj_uniform_1, _obj_uniform_2) in components.query2d::<ObjectUniform1::FromComponent, ObjectUniform2::FromComponent>() {
            let mesh_id = ObjectUniform1::mesh_id(obj_uniform_1);
            self.rendering_map.add_instance(mesh_id, entity_id);
        }
        // add offsets to the map
        // O(n) complexity
        self.rendering_map.add_offsets();
    }

    fn assert_uniform_buffer_sizes(
        &mut self,
        image_index: usize,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
    ) -> PResult<()> {
        let object_count = self.rendering_map.object_count();
        self.frame_1_uniform_buffer.assert_buffer_size(1, image_index, vk_instance, vk_device, vk_physical_device)?;
        self.frame_2_uniform_buffer.assert_buffer_size(1, image_index, vk_instance, vk_device, vk_physical_device)?;
        self.frame_3_uniform_buffer.assert_buffer_size(1, image_index, vk_instance, vk_device, vk_physical_device)?;
        self.object_1_uniform_buffer.assert_buffer_size(object_count, image_index, vk_instance, vk_device, vk_physical_device)?;
        self.object_2_uniform_buffer.assert_buffer_size(object_count, image_index, vk_instance, vk_device, vk_physical_device)?;

        Ok(())
    }

    fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        self.creation_state.destroy(vk_device);
        self.frame_1_uniform_buffer.destroy_buffer(vk_device);
        self.frame_2_uniform_buffer.destroy_buffer(vk_device);
        self.frame_3_uniform_buffer.destroy_buffer(vk_device);
        self.object_1_uniform_buffer.destroy_buffer(vk_device);
        self.object_2_uniform_buffer.destroy_buffer(vk_device);
        unsafe {
            vk_device.destroy_descriptor_pool(self.vk_descriptor_pool, None);
            vk_device.destroy_pipeline(self.pipeline, None);
            vk_device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}


