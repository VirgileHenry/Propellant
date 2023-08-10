

/*
How to create pipelines: 

create_graphic_pipeline!(
    (VERTEX_SHADER_CODE, stage::Vertex),
    (FRAGMENT_SHADER_CODE, stage::Fragment);
    (FrameUniform uniforms::FrameUniform1 stage::Vertex), // set 0
    (FrameUniform uniforms::FrameUniform2 stage::Fragment), // set 1
    (ResourceUniform uniforms::FrameUniform3 stage::Fragment), // set 2
    (RenderableUniform uniforms::ObjectUniform1 stage::Fragment), // set 3
    (ObjectUniform uniforms::ObjectUniform2 stage::Vertex), // set4
);

this should geneate a builder and the pipeline, implementing the according traits for them.

*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl Into<vulkanalia::vk::ShaderStageFlags> for ShaderStage {
    fn into(self) -> vulkanalia::vk::ShaderStageFlags {
        match self {
            ShaderStage::Vertex => vulkanalia::vk::ShaderStageFlags::VERTEX,
            ShaderStage::Fragment => vulkanalia::vk::ShaderStageFlags::FRAGMENT,
        }
    }
}

#[macro_export]
macro_rules! create_graphic_pipeline_impl {
    // ========== Recursive call to build the fields properly ==========
    (
        $(($shader_stage:path, $shader_code:ident)),*;
        ($($frm_uniforms_decl:tt)*); ($($obj_uniforms_decl:tt)*); ($($rc_uniforms_decl:tt)*);
        ($($frm_buffers_decl:tt)*); ($($obj_buffers_decl:tt)*); ($($rc_buffers_decl:tt)*);
        ($($frm_uniforms_field:tt)*); ($($obj_uniforms_field:tt)*); ($($rc_uniforms_field:tt)*);
        ($($frm_uniforms_build:tt)*); ($($obj_uniforms_build:tt)*); ($($rc_uniforms_build:tt)*);
        ($($frm_uniforms_type:tt)*); ($($obj_uniforms_type:tt)*); ($($rc_uniforms_type:tt)*);
        (FrameUniform, $uniform:ident, $stage:path),
        $($rest:tt)*
    ) => {
        // register a new frame uniform
        crate::create_graphic_pipeline_impl!(
            $(($shader_stage, $shader_code)),*;
            ($($frm_uniforms_decl)* [<frame_uniform_ $uniform:snake>]: UniformBufferBuilder<$uniform>,); ($($obj_uniforms_decl)*); ($($rc_uniforms_decl)*);
            ($($frm_buffers_decl)* [<frame_uniform_ $uniform:snake>]: UniformBuffer<$uniform>,); ($($obj_buffers_decl)*); ($($rc_buffers_decl)*);
            ($($frm_uniforms_field)* [<frame_uniform_ $uniform:snake>]); ($($obj_uniforms_field)*); ($($rc_uniforms_field)*);
            ($($frm_uniforms_build)* [<frame_uniform_ $uniform:snake>]: UniformBufferBuilder::new($stage.into(), vulkanalia::vk::DescriptorType::UNIFORM_BUFFER),); ($($obj_uniforms_build)*); ($($rc_uniforms_build)*);
            ($($frm_uniforms_type)* $uniform); ($($obj_uniforms_type)*); ($($rc_uniforms_type)*);
            $($rest)*
        )
    };
    (
        $(($shader_stage:path, $shader_code:ident)),*;
        ($($frm_uniforms_decl:tt)*); ($($obj_uniforms_decl:tt)*); ($($rc_uniforms_decl:tt)*);
        ($($frm_buffers_decl:tt)*); ($($obj_buffers_decl:tt)*); ($($rc_buffers_decl:tt)*);
        ($($frm_uniforms_field:tt)*); ($($obj_uniforms_field:tt)*); ($($rc_uniforms_field:tt)*);
        ($($frm_uniforms_build:tt)*); ($($obj_uniforms_build:tt)*); ($($rc_uniforms_build:tt)*);
        ($($frm_uniforms_type:tt)*); ($($obj_uniforms_type:tt)*); ($($rc_uniforms_type:tt)*);
        (ObjectUniform, $uniform:ident, $stage:path),
        $($rest:tt)*
    ) => {
        // register a new renderable object uniform
        crate::create_graphic_pipeline_impl!(
            $(($shader_stage, $shader_code)),*;
            ($($frm_uniforms_decl)*); ($($obj_uniforms_decl)* [<object_uniform_ $uniform:snake>]: UniformBufferBuilder<$uniform>,); ($($rc_uniforms_decl)*);
            ($($frm_buffers_decl)*); ($($obj_buffers_decl)* [<object_uniform_ $uniform:snake>]: UniformBuffer<$uniform>,); ($($rc_buffers_decl)*);
            ($($frm_uniforms_field)*); ($($obj_uniforms_field)* [<object_uniform_ $uniform:snake>]); ($($rc_uniforms_field)*);
            ($($frm_uniforms_build)*); ($($obj_uniforms_build)* [<object_uniform_ $uniform:snake>]: UniformBufferBuilder::new($stage.into(), vulkanalia::vk::DescriptorType::STORAGE_BUFFER),); ($($rc_uniforms_build)*);
            ($($frm_uniforms_type)*); ($($obj_uniforms_type)* $uniform); ($($rc_uniforms_type)*);
            $($rest)*
        )
    };
    (
        $(($shader_stage:path, $shader_code:ident)),*;
        ($($frm_uniforms_decl:tt)*); ($($obj_uniforms_decl:tt)*); ($($rc_uniforms_decl:tt)*);
        ($($frm_buffers_decl:tt)*); ($($obj_buffers_decl:tt)*); ($($rc_buffers_decl:tt)*);
        ($($frm_uniforms_field:tt)*); ($($obj_uniforms_field:tt)*); ($($rc_uniforms_field:tt)*);
        ($($frm_uniforms_build:tt)*); ($($obj_uniforms_build:tt)*); ($($rc_uniforms_build:tt)*);
        ($($frm_uniforms_type:tt)*); ($($obj_uniforms_type:tt)*); ($($rc_uniforms_type:tt)*);
        (RenderableComponent, $uniform:ident, $stage:path),
        $($rest:tt)*
    ) => {
        // register a new renderable object uniform
        {
            crate::create_graphic_pipeline_impl!(
                $(($shader_stage, $shader_code)),*;
                ($($frm_uniforms_decl)*); ($($obj_uniforms_decl)*); ($($rc_uniforms_decl)* [<renderable_comp_uniform_ $uniform:snake>]: UniformBufferBuilder<$uniform>,);
                ($($frm_buffers_decl)*); ($($obj_buffers_decl)*); ($($rc_buffers_decl)* [<renderable_comp_uniform_ $uniform:snake>]: UniformBuffer<$uniform>,);
                ($($frm_uniforms_field)*); ($($obj_uniforms_field)*); ($($rc_uniforms_field)* [<renderable_comp_uniform_ $uniform:snake>]);
                ($($frm_uniforms_build)*); ($($obj_uniforms_build)*); ($($rc_uniforms_build)* [<renderable_comp_uniform_ $uniform:snake>]: UniformBufferBuilder::new($stage.into(), vulkanalia::vk::DescriptorType::STORAGE_BUFFER),);
                ($($frm_uniforms_type)*); ($($obj_uniforms_type)*); ($($rc_uniforms_type)* $uniform);
                $($rest)*
            )
        }
    };
    // ========== Macro expansion with properly built fields ==========
    (
        $(($shader_stage:path, $shader_code:ident)),*;
        ($($frm_uniforms_decl:tt)*); ($($obj_uniforms_decl:tt)*); ($($rc_uniforms_decl:tt)*);
        ($($frm_buffers_decl:tt)*); ($($obj_buffers_decl:tt)*); ($($rc_buffers_decl:tt)*);
        ($($frm_uniforms_field:tt)*); ($($obj_uniforms_field:tt)*); ($($rc_uniforms_field:tt)*);
        ($($frm_uniforms_build:tt)*); ($($obj_uniforms_build:tt)*); ($($rc_uniforms_build:tt)*);
        ($($frm_uniforms_type:tt)*); ($($obj_uniforms_type:tt)*); ($($rc_uniforms_type:tt)*);
    ) => {
        { paste::paste! { // we filled our tt with paste syntax, time to unpack it

            // ========== Create the pipeline struct ==========
            use vulkanalia::vk::Handle;

            pub struct GraphicPipeline {
                pipeline: vulkanalia::vk::Pipeline,
                pipeline_layout: vulkanalia::vk::PipelineLayout,
                vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
                creation_state: crate::engine::renderer::graphic_pipeline::GraphicPipelineCreationState,
                rendering_map: crate::engine::renderer::rendering_map::RenderingMap,
                $(
                    $frm_buffers_decl
                )*
                $(
                    $obj_buffers_decl
                )*
                $(
                    $rc_buffers_decl
                )*
            }

            impl GraphicPipeline {
                fn create(
                    vk_device: &vulkanalia::Device,
                    vertex_binding_description: Vec<vulkanalia::vk::VertexInputBindingDescription>,
                    vertex_attribute_description: Vec<vulkanalia::vk::VertexInputAttributeDescription>,
                    shader_stages: HashMap<vulkanalia::vk::ShaderStageFlags, vulkanalia::vk::ShaderModule>,
                    swapchain_extent: vulkanalia::vk::Extent2D,
                    pipeline_layout: vulkanalia::vk::PipelineLayout,
                    render_pass: vulkanalia::vk::RenderPass,
                    vk_descriptor_pool: vulkanalia::vk::DescriptorPool,
                    $($frm_buffers_decl)*
                    $($obj_buffers_decl)*
                    $($rc_buffers_decl)*
                ) -> PResult<GraphicPipeline> {
            
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
                        
                    Ok(GraphicPipeline {
                        $($frm_uniforms_field,)*
                        $($obj_uniforms_field,)*
                        $($rc_uniforms_field,)*
                        pipeline,
                        pipeline_layout,
                        vk_descriptor_pool,
                        creation_state,
                        rendering_map: crate::engine::renderer::rendering_map::RenderingMap::new(),
                    })
                }
            }

            impl GraphicPipelineInterface for GraphicPipeline  {
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
                    $(self.$frm_uniforms_field.map(vk_device, image_index)?;)*
                    $(self.$obj_uniforms_field.map(vk_device, image_index)?;)*
                    $(self.$rc_uniforms_field.map(vk_device, image_index)?;)*
                    // frame uniforms
                    $(
                        self.$frm_uniforms_field.update_buffer(0, image_index, $frm_uniforms_type::get_uniform(components));
                    )*
                    // object uniforms
                    // TODO : hard coded query 2D here, but this depends on the number of object + resources uniforms.
                    for (
                        _,
                        $($rc_uniforms_field,)*
                        $($obj_uniforms_field,)*                    
                    ) in components.query2d::<
                        $(<$rc_uniforms_type as ObjectUniform>::FromComponent,)*
                        $(<$obj_uniforms_type as ObjectUniform>::FromComponent,)*
                    >() {
                        $(let uniform_buffer_offset = <$rc_uniforms_type as RenderableComponent>::uniform_buffer_index($rc_uniforms_field);)*
                        $(self.$rc_uniforms_field.update_buffer(uniform_buffer_offset, image_index, <$rc_uniforms_type as ObjectUniform>::get_uniform($rc_uniforms_field));)*
                        $(self.$obj_uniforms_field.update_buffer(uniform_buffer_offset, image_index, <$obj_uniforms_type as ObjectUniform>::get_uniform($obj_uniforms_field));)*
                    }
                    // unmap all the buffers
                    $(self.$frm_uniforms_field.unmap(vk_device, image_index);)*
                    $(self.$obj_uniforms_field.unmap(vk_device, image_index);)*
                    $(self.$rc_uniforms_field.unmap(vk_device, image_index);)*
                
                    Ok(())
                }
            
                fn register_draw_commands(
                    &self,
                    vk_device: &vulkanalia::Device,
                    image_index: usize,
                    command_buffer: vulkanalia::vk::CommandBuffer,
                    resources: &crate::ProppellantResources,
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
                        $(self.$frm_uniforms_field.set(image_index),)*
                        $(self.$obj_uniforms_field.set(image_index),)*
                        $(self.$rc_uniforms_field.set(image_index),)*
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
                                instance_count as u32,
                                0,
                                0,
                                first_instance as u32
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
                    let map = self.rendering_map.map_mut();
                    // clear the map
                    map.clear();
                    // iterate over objects, count how many for each mesh
                    // O(n) complexity
                    for (
                        _,
                        $($rc_uniforms_field,)*
                        $($obj_uniforms_field,)*                    
                    ) in components.query2d::<
                        $(<$rc_uniforms_type as ObjectUniform>::FromComponent,)*
                        $(<$obj_uniforms_type as ObjectUniform>::FromComponent,)*
                    >() {
                        $(
                            match map.get_mut(&<$rc_uniforms_type as RenderableComponent>::mesh_id($rc_uniforms_field)) {
                                Some((instance_count, _, _)) => *instance_count += 1,
                                None => {map.insert(<$rc_uniforms_type as RenderableComponent>::mesh_id($rc_uniforms_field), (1, 0, 0));},
                            }
                        )*
                    }
                    // add offsets to the map
                    let mut offset = 0;
                    for (_, (instance_count, total_offset, counter)) in map.iter_mut() {
                        *total_offset = offset;
                        offset += *instance_count;
                        *counter = 0;
                    }
                    // final loop to set the buffers offsets
                    for (
                        _,
                        $($rc_uniforms_field,)*
                        $($obj_uniforms_field,)*                    
                    ) in components.query2d_mut::<
                        $(<$rc_uniforms_type as ObjectUniform>::FromComponent,)*
                        $(<$obj_uniforms_type as ObjectUniform>::FromComponent,)*
                    >() {
                        $(
                            let (_, mesh_offset, counter) = map.get_mut(&<$rc_uniforms_type as RenderableComponent>::mesh_id($rc_uniforms_field)).unwrap();
                            <$rc_uniforms_type as RenderableComponent>::set_uniform_buffer_index($rc_uniforms_field, *mesh_offset + *counter);
                            *counter += 1;
                        )*
                    }
                }
            
                fn assert_uniform_buffer_sizes(
                    &mut self,
                    image_index: usize,
                    vk_instance: &vulkanalia::Instance,
                    vk_device: &vulkanalia::Device,
                    vk_physical_device: vulkanalia::vk::PhysicalDevice,
                ) -> PResult<()> {
                    let object_count = self.rendering_map.object_count();

                    $(self.$frm_uniforms_field.assert_buffer_size(1, image_index, vk_instance, vk_device, vk_physical_device)?;)*
                    $(self.$obj_uniforms_field.assert_buffer_size(object_count, image_index, vk_instance, vk_device, vk_physical_device)?;)*
                    $(self.$rc_uniforms_field.assert_buffer_size(object_count, image_index, vk_instance, vk_device, vk_physical_device)?;)*
                
                    Ok(())
                }
            
                fn destroy(
                    &mut self,
                    vk_device: &vulkanalia::Device,
                ) {
                    self.creation_state.destroy(vk_device);
                    $(self.$frm_uniforms_field.destroy_buffer(vk_device);)*
                    $(self.$obj_uniforms_field.destroy_buffer(vk_device);)*
                    $(self.$rc_uniforms_field.destroy_buffer(vk_device);)*
                    unsafe {
                        vk_device.destroy_descriptor_pool(self.vk_descriptor_pool, None);
                        vk_device.destroy_pipeline(self.pipeline, None);
                        vk_device.destroy_pipeline_layout(self.pipeline_layout, None);
                    }
                }
            }

            // ========== Create the builder struct ==========

            pub struct GraphicPipelineBuilder {
                shaders: std::collections::HashMap<crate::ShaderStage, Vec<u32>>,
                $($frm_uniforms_decl)*
                $($obj_uniforms_decl)*
                $($rc_uniforms_decl)*
            }

            impl GraphicPipelineBuilder {
                pub fn build_inner(
                    self,
                    vk_device: &vulkanalia::Device,
                    swapchain_extent: vulkanalia::vk::Extent2D,
                    swapchain_image_count: usize,
                    render_pass: vulkanalia::vk::RenderPass
                ) -> PResult<GraphicPipeline> {
                    // create shader modules (compile byte code)
                    let shader_stages = self.shaders.iter().map(|(stage, code)| {
                        create_shader_module(code, vk_device).and_then(|shader| Ok(((*stage).into(), shader)))
                    }).collect::<Result<std::collections::HashMap<_, _>, _>>()?;

                    let descriptor_types = vec![
                        $((self.$frm_uniforms_field).descriptor_type(),)*
                        $((self.$obj_uniforms_field).descriptor_type(),)*
                        $((self.$rc_uniforms_field).descriptor_type(),)*
                    ];
            
                    // create the descriptor pool, to allocate descriptor sets.
                    let vk_descriptor_pool = create_descriptor_pool(
                        vk_device,
                        descriptor_types,
                        swapchain_image_count
                    )?;
            
                    // create the uniforms
                    $(let $frm_uniforms_field = self.$frm_uniforms_field.build(vk_device, vk_descriptor_pool, swapchain_image_count)?;)*
                    $(let $obj_uniforms_field = self.$obj_uniforms_field.build(vk_device, vk_descriptor_pool, swapchain_image_count)?;)*
                    $(let $rc_uniforms_field = self.$rc_uniforms_field.build(vk_device, vk_descriptor_pool, swapchain_image_count)?;)*

                    let layouts = vec![
                        $($frm_uniforms_field.layout(),)*
                        $($obj_uniforms_field.layout(),)*
                        $($rc_uniforms_field.layout(),)*
                    ];
                    
                    // pipeline layout is where we set all our uniforms declaration
                    let layout_info = vulkanalia::vk::PipelineLayoutCreateInfo::builder()
                        .set_layouts(&layouts);
            
                    // create the pipeline layout and the pipeline.
                    let pipeline_layout = unsafe { vk_device.create_pipeline_layout(&layout_info, None)? };
                    
                    // set the vertex input state
                    let vertex_binding_description = vec![Vertex::binding_description()];
                    let vertex_attribute_description = Vertex::attribute_description();
                    
                    GraphicPipeline::create(
                        vk_device, 
                        vertex_binding_description,
                        vertex_attribute_description,
                        shader_stages,
                        swapchain_extent,
                        pipeline_layout,
                        render_pass,
                        vk_descriptor_pool,
                        $($frm_uniforms_field,)*
                        $($obj_uniforms_field,)*
                        $($rc_uniforms_field,)*
                    )
                }
            }

            impl GraphicPipelineBuilderInterface for GraphicPipelineBuilder {
                fn build(
                    self: Box<Self>,
                    vk_device: &vulkanalia::Device,
                    swapchain_extent: vulkanalia::vk::Extent2D,
                    frame_count: usize,
                    render_pass: vulkanalia::vk::RenderPass,
                ) -> PResult<Box<dyn GraphicPipelineInterface>> {
                    Ok(Box::new(self.build_inner(vk_device, swapchain_extent, frame_count, render_pass)?))
                }
            }

            GraphicPipelineBuilder {
                shaders: vec![
                    $(
                        ($shader_stage, $shader_code.to_vec()),
                    )*
                ].into_iter().collect(),
                $($frm_uniforms_build)*
                $($obj_uniforms_build)*
                $($rc_uniforms_build)*
            }
        } }
    }
}

#[macro_export]
macro_rules! create_graphic_pipeline {
    (
        $(($shader_stage:path, $shader_code:ident)),*;
        $($uniform_data:tt)*
    ) => {
        crate::create_graphic_pipeline_impl!(
            $(($shader_stage, $shader_code)),*;
            (); (); (); // uniforms declaration
            (); (); (); // built buffers declaration
            (); (); (); // uniforms field
            (); (); (); // uniforms build
            (); (); (); // uniforms types
            $($uniform_data)*
        )
    };
}
