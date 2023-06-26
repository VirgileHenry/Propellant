use std::collections::HashMap;
use std::fmt::Debug;

use crate::engine::errors::PResult;
use crate::engine::material::phong_material::PhongMaterialProperties;
use crate::engine::mesh::vertex::Vertex;
use crate::engine::renderer::shaders::DEFAULT_FRAG;
use crate::engine::renderer::shaders::DEFAULT_VERT;

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::Handle;

use super::GraphicsPipeline;
use super::uniform::frame_uniform::AsPerFrameUniform;
use super::uniform::frame_uniform::FrameUniformBuilder;
use super::uniform::frame_uniform::camera_uniform::CameraUniformObject;
use super::uniform::frame_uniform::main_directionnal_light::MainDirectionnalLight;
use super::uniform::object_uniform::AsPerObjectUniform;
use super::uniform::object_uniform::ObjectUniformBuilder;
use super::uniform::object_uniform::model_uniform::ModelMatrixUniformObject;
use super::uniform::resource_uniform::ResourceUniformBuilder;
use super::uniform::resource_uniform::textures_uniform::TextureUniformBuilder;
use super::uniform::uniform_buffer::UniformBufferBuilder;


/// defines the rendering process. Must be given to the vulkan interface to be built.
/// todo : add full shader types support.
#[derive(Debug)]
pub struct GraphicsPipelineBuilder {
    /// Vertex shader byte code
    vertex: (Vec<u32>, usize),
    /// Fragment shader byte code
    fragment: (Vec<u32>, usize),
    /// Per resources uniforms
    resource_uniforms: Vec<Box<dyn ResourceUniformBuilder>>,
    /// Per frames uniforms
    frame_uniforms: Vec<Box<dyn FrameUniformBuilder>>,
    /// Per object uniforms
    object_uniforms: Vec<Box<dyn ObjectUniformBuilder>>,
}

impl GraphicsPipelineBuilder {
    pub fn empty() -> GraphicsPipelineBuilder {
        GraphicsPipelineBuilder {
            vertex: (Vec::with_capacity(0), 0),
            fragment: (Vec::with_capacity(0), 0),
            resource_uniforms: Vec::new(),
            frame_uniforms: Vec::new(),
            object_uniforms: Vec::new(),
        }
    }

    pub fn build(
        &self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        swapchain_extent: vulkanalia::vk::Extent2D,
        swapchain_images: &[vulkanalia::vk::Image],
        render_pass: vulkanalia::vk::RenderPass
    ) -> PResult<GraphicsPipeline> {
        // create shader modules (compile byte code)
        let vert_shader_module = Self::create_shader_module((&self.vertex.0, self.vertex.1), vk_device)?;
        let frag_shader_module = Self::create_shader_module((&self.fragment.0, self.fragment.1), vk_device)?;
        
        
        // create the staging for the shaders
        let vert_stage = vulkanalia::vk::PipelineShaderStageCreateInfo::builder()
            .stage(vulkanalia::vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vulkanalia::vk::PipelineShaderStageCreateInfo::builder()
            .stage(vulkanalia::vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");
        
        // set the vertex input state
        let vertex_binding_description = [Vertex::binding_description()];
        let vertex_attribute_description = Vertex::attribute_description();
        let vertex_input_state = vulkanalia::vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_binding_description)
            .vertex_attribute_descriptions(&vertex_attribute_description);
        
        // here, default values to draw triangles. Maybe to rework at some point ? 
        let input_assembly_state = vulkanalia::vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vulkanalia::vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        
        
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
        
        // create the viewport state
        
        
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
            .depth_bias_enable(false);
        
        // multisampling state: antialiasing here
        let multisample_state = vulkanalia::vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vulkanalia::vk::SampleCountFlags::_1);
        
        // color blending. transparency and alpha color blending can be done here !
        let color_attachment = vulkanalia::vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vulkanalia::vk::ColorComponentFlags::all())
            .blend_enable(false);

        // todo : depth attachment 

        let attachments = &[color_attachment];
        let color_blend_state = vulkanalia::vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vulkanalia::vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);
        
        // create the descriptor pool, to allocate descriptor sets.
        let vk_descriptor_pool = self.create_descriptor_pool(vk_device, swapchain_images)?;

        // create the uniforms
        let resource_uniforms = self.resource_uniforms.iter().map(|builder|
            builder.build(vk_device, vk_descriptor_pool)
        ).collect::<PResult<Vec<_>>>()?;

        let frame_uniforms = self.frame_uniforms.iter().map(|builder|
            builder.build(vk_device, vk_descriptor_pool, swapchain_images.len())
        ).collect::<PResult<Vec<_>>>()?;

        let object_uniforms = self.object_uniforms.iter().map(|builder|
            builder.build(vk_device, vk_descriptor_pool, swapchain_images.len())
        ).collect::<PResult<Vec<_>>>()?;

        let empty_ds: Vec<vulkanalia::vk::DescriptorSetLayout> = Vec::with_capacity(0);

        // get the layout of the uniforms
        // ! this SHOULD be ordered by set.
        let layouts = empty_ds.into_iter()
            .chain(resource_uniforms.iter().map(|uniform| uniform.layout()))
            .chain(frame_uniforms.iter().map(|uniform| uniform.layout()))
            .chain(object_uniforms.iter().map(|uniform| uniform.layout()))
            .collect::<Vec<_>>();
        
        // pipeline layout is where we set all our uniforms declaration
        let layout_info = vulkanalia::vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&layouts);

        // create the pipeline layout and the pipeline.
        let pipeline_layout = unsafe { vk_device.create_pipeline_layout(&layout_info, None)? };
        

        // create the pipeline ! 
        let stages = &[vert_stage, frag_stage];
        
        let info = vulkanalia::vk::GraphicsPipelineCreateInfo::builder()
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_handle(vulkanalia::vk::Pipeline::null()) // Optional.
            .base_pipeline_index(-1); // Optional.

        let pipeline = unsafe {
            vk_device.create_graphics_pipelines(vulkanalia::vk::PipelineCache::null(), &[info], None)?.0
        };

        // we no longer need the shader module, clean them up !
        unsafe {
            vk_device.destroy_shader_module(vert_shader_module, None);
            vk_device.destroy_shader_module(frag_shader_module, None);
        }

        Ok(GraphicsPipeline::new(
            pipeline,
            pipeline_layout,
            vk_descriptor_pool,
            resource_uniforms,
            frame_uniforms,
            object_uniforms,
        ))
    }

    fn create_shader_module(source_code: (&[u32], usize), vk_device: &vulkanalia::Device) -> PResult<vulkanalia::vk::ShaderModule> {
        let info = vulkanalia::vk::ShaderModuleCreateInfo::builder()
            .code_size(source_code.1)
            .code(source_code.0);

        Ok(unsafe { vk_device.create_shader_module(&info, None)? })
    }

    fn create_descriptor_pool(
        &self,
        vk_device: &vulkanalia::Device,
        swapchain_images: &[vulkanalia::vk::Image],
    ) -> PResult<vulkanalia::vk::DescriptorPool> {
        let descriptor_set_count = (
            self.resource_uniforms.len() +
            self.frame_uniforms.len() +
            self.object_uniforms.len()
        ) * swapchain_images.len();

        // for each layout type, we count how many descriptor sets we need.
        let mut ds_count_map = HashMap::with_capacity(3);
        // resources uniforms
        self.resource_uniforms.iter().for_each(|builder| {
            match ds_count_map.get_mut(&builder.descriptor_type()) {
                Some(count) => *count += swapchain_images.len(),
                None => { ds_count_map.insert(builder.descriptor_type(), swapchain_images.len()); },
            }
        });
        // frame uniforms
        self.frame_uniforms.iter().for_each(|builder| {
            match ds_count_map.get_mut(&builder.descriptor_type()) {
                Some(count) => *count += swapchain_images.len(),
                None => { ds_count_map.insert(builder.descriptor_type(), swapchain_images.len()); },
            }
        });
        // object uniforms
        self.object_uniforms.iter().for_each(|builder| {
            match ds_count_map.get_mut(&builder.descriptor_type()) {
                Some(count) => *count += swapchain_images.len(),
                None => { ds_count_map.insert(builder.descriptor_type(), swapchain_images.len()); },
            }
        });
        
        let pool_sizes = ds_count_map.into_iter().map(|(ds_type, count)| {
            vulkanalia::vk::DescriptorPoolSize::builder()
                .type_(ds_type)
                .descriptor_count(count as u32)
        }).collect::<Vec<_>>();

        let info = vulkanalia::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(descriptor_set_count as u32);

        Ok( unsafe { 
            vk_device.create_descriptor_pool(&info, None)?
        })
    }

    pub fn with_frame_uniform<T: AsPerFrameUniform + Debug + 'static>(
        &mut self,
        stage: vulkanalia::vk::ShaderStageFlags,
    ) {
        // add the uniform builder to the list. 
        // use the current length of the uniforms as a binding, so they respect their index.
        self.frame_uniforms.push(Box::new(UniformBufferBuilder::<T>::new(
            stage,
            vulkanalia::vk::DescriptorType::UNIFORM_BUFFER, // per frame uniforms uses uniform buffers
            self.frame_uniforms.len() as u32
        )));
    }


    pub fn with_object_uniform<T: AsPerObjectUniform + Debug + 'static>(
        &mut self,
        stage: vulkanalia::vk::ShaderStageFlags,
    ) {
        // add the uniform builder to the list. 
        // use the current length of the uniforms as a binding, so they respect their index.
        self.object_uniforms.push(Box::new(UniformBufferBuilder::<T>::new(
            stage,
            vulkanalia::vk::DescriptorType::STORAGE_BUFFER, // per object uniforms uses storage buffers
            self.object_uniforms.len() as u32
        )));
    }

}

impl Default for GraphicsPipelineBuilder {
    /// Default rendering pipeline.
    fn default() -> Self {
        
        // return the builder
        GraphicsPipelineBuilder { 
            vertex: (DEFAULT_VERT.iter().map(|v| *v).collect(), DEFAULT_VERT.len() * 4), // x4 because we are using u32, and length is in byte
            fragment: (DEFAULT_FRAG.iter().map(|v| *v).collect(), DEFAULT_FRAG.len() * 4), // x4 because we are using u32, and length is in byte
            resource_uniforms: vec![
                Box::new(TextureUniformBuilder::new(0, vulkanalia::vk::ShaderStageFlags::FRAGMENT))
            ],
            frame_uniforms: vec![
                Box::new(UniformBufferBuilder::<CameraUniformObject>::new(vulkanalia::vk::ShaderStageFlags::VERTEX, vulkanalia::vk::DescriptorType::UNIFORM_BUFFER, 0)),
                Box::new(UniformBufferBuilder::<MainDirectionnalLight>::new(vulkanalia::vk::ShaderStageFlags::FRAGMENT, vulkanalia::vk::DescriptorType::UNIFORM_BUFFER, 0)),
            ],
            object_uniforms: vec![
                Box::new(UniformBufferBuilder::<ModelMatrixUniformObject>::new(vulkanalia::vk::ShaderStageFlags::VERTEX, vulkanalia::vk::DescriptorType::STORAGE_BUFFER, 0)),
                Box::new(UniformBufferBuilder::<PhongMaterialProperties>::new(vulkanalia::vk::ShaderStageFlags::FRAGMENT, vulkanalia::vk::DescriptorType::STORAGE_BUFFER, 0)),
            ],
        }
    }
}