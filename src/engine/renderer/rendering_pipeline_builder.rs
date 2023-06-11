use crate::engine::mesh::vertex::Vertex;
use crate::engine::{renderer::shaders::DEFAULT_FRAG, errors::PropellantError};
use super::rendering_pipeline::RenderingPipeline;
use super::rendering_pipeline::camera_uniform::camera_uniform_generator;
use super::rendering_pipeline::uniform_descriptor_set::per_frame_uniform::PerFrameUniformObject;
use super::rendering_pipeline::uniform_descriptor_set::per_frame_uniform_builder::PerFrameUniformBuilder;
use super::shaders::DEFAULT_VERT;

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::Handle;


/// defines the rendering process. Must be given to the vulkan interface to be built.
/// todo : add full shader types support.
#[derive(Debug)]
pub struct RenderingPipelineBuilder {
    /// Vertex shader byte code
    vertex: (Vec<u32>, usize),
    /// Fragment shader byte code
    fragment: (Vec<u32>, usize),
    /// per frames uniforms.
    per_frame_uniforms: Vec<PerFrameUniformBuilder>
}

impl RenderingPipelineBuilder {
    pub fn empty() -> RenderingPipelineBuilder {
        RenderingPipelineBuilder {
            vertex: (Vec::with_capacity(0), 0),
            fragment: (Vec::with_capacity(0), 0),
            per_frame_uniforms: Vec::with_capacity(0),
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
    ) -> Result<RenderingPipeline, PropellantError> {
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
        
        
        // create a default sized viewport
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
        
        // todo : depth buffer set up !
        // color blending. transparency and alpha color blending can be done here !
        let attachment = vulkanalia::vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vulkanalia::vk::ColorComponentFlags::all())
            .blend_enable(false);
        let attachments = &[attachment];
        let color_blend_state = vulkanalia::vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vulkanalia::vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);
        
        // create the descriptor pool, to allocate descriptor sets.
        let descriptor_pool = self.create_descriptor_pool(vk_device, swapchain_images)?;

        let per_frame_uniforms = self.per_frame_uniforms.iter().map(|builder| {
            PerFrameUniformObject::new(
                builder,
                vk_instance,
                vk_device,
                vk_physical_device,
                descriptor_pool,
                swapchain_images.len(),
            )
        }).collect::<Result<Vec<_>, _>>()?;

        let layouts = per_frame_uniforms.iter().map(|uniform| uniform.layout()).collect::<Vec<_>>();
        
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

        Ok(RenderingPipeline::new(
            pipeline,
            pipeline_layout,
            per_frame_uniforms,
            descriptor_pool,
        ))
    }

    fn create_shader_module(source_code: (&[u32], usize), vk_device: &vulkanalia::Device) -> Result<vulkanalia::vk::ShaderModule, PropellantError> {
        let info = vulkanalia::vk::ShaderModuleCreateInfo::builder()
            .code_size(source_code.1)
            .code(source_code.0);

        Ok(unsafe { vk_device.create_shader_module(&info, None)? })
    }


    fn create_descriptor_pool(
        &self,
        vk_device: &vulkanalia::Device,
        swapchain_images: &[vulkanalia::vk::Image],
    ) -> Result<vulkanalia::vk::DescriptorPool, PropellantError> {
        
        let ubo_size = vulkanalia::vk::DescriptorPoolSize::builder()
            .type_(vulkanalia::vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(swapchain_images.len() as u32);

        let pool_sizes = &[ubo_size];
        let info = vulkanalia::vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(swapchain_images.len() as u32);

        Ok( unsafe { 
            vk_device.create_descriptor_pool(&info, None)?
        })
    }

}

impl Default for RenderingPipelineBuilder {
    fn default() -> Self {
        
        // return the builder
        RenderingPipelineBuilder { 
            vertex: (DEFAULT_VERT.iter().map(|v| *v).collect(), DEFAULT_VERT.len() * 4),
            fragment: (DEFAULT_FRAG.iter().map(|v| *v).collect(), DEFAULT_FRAG.len() * 4),
            per_frame_uniforms: vec![
                PerFrameUniformBuilder::new(|comps| camera_uniform_generator(comps)),
            ],
        }
    }
}