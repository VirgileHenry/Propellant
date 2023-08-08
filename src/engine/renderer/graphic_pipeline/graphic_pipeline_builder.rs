use std::collections::HashMap;

use crate::CameraUniformObject;
use crate::MainDirectionnalLight;
use crate::ModelMatrixUniformObject;
use crate::PhongMaterial;
use crate::engine::errors::PResult;
use crate::engine::mesh::vertex::Vertex;
use crate::engine::renderer::shaders::DEFAULT_FRAG;
use crate::engine::renderer::shaders::DEFAULT_VERT;

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;

use super::GraphicPipelineFFFOO;
use super::GraphicPipelineInterface;
use super::renderable_component::RenderableComponent;
use super::uniform::frame_uniform::FrameUniform;
use super::uniform::object_uniform::ObjectUniform;
use super::uniform::uniform_buffer::UniformBufferBuilder;

pub trait GraphicPipelineBuilderInterface {
    fn build(
        self: Box<Self>,
        vk_device: &vulkanalia::Device,
        swapchain_extent: vulkanalia::vk::Extent2D,
        frame_count: usize,
        render_pass: vulkanalia::vk::RenderPass,
    ) -> PResult<Box<dyn GraphicPipelineInterface>>; 
}


fn create_shader_module(source_code: &[u32], vk_device: &vulkanalia::Device) -> PResult<vulkanalia::vk::ShaderModule> {
    let info = vulkanalia::vk::ShaderModuleCreateInfo::builder()
        .code_size(source_code.len() * 4)
        .code(source_code); // x4 because we are using u32, and length is in byte

    Ok(unsafe { vk_device.create_shader_module(&info, None)? })
}

pub struct GraphicPipelineBuilderFFFOO<
    FrameUniform1: FrameUniform,
    FrameUniform2: FrameUniform,
    FrameUniform3: FrameUniform,
    ObjectUniform1: RenderableComponent,
    ObjectUniform2: ObjectUniform
> {
    phantom: std::marker::PhantomData<(FrameUniform1, FrameUniform2, FrameUniform3, ObjectUniform1, ObjectUniform2)>,
    vertex_shader: Vec<u32>,
    fragment_shader: Vec<u32>,
    frame_uniform_1_buffer_builder: UniformBufferBuilder<FrameUniform1>,
    frame_uniform_2_buffer_builder: UniformBufferBuilder<FrameUniform2>,
    frame_uniform_3_buffer_builder: UniformBufferBuilder<FrameUniform3>,
    object_uniform_1_buffer_builder: UniformBufferBuilder<ObjectUniform1>,
    object_uniform_2_buffer_builder: UniformBufferBuilder<ObjectUniform2>,
}

impl<
    FrameUniform1: FrameUniform + 'static,
    FrameUniform2: FrameUniform + 'static,
    FrameUniform3: FrameUniform + 'static,
    ObjectUniform1: RenderableComponent + 'static,
    ObjectUniform2: ObjectUniform + 'static,
> GraphicPipelineBuilderFFFOO<FrameUniform1, FrameUniform2, FrameUniform3, ObjectUniform1, ObjectUniform2> {
    pub fn build_inner(
        self,
        vk_device: &vulkanalia::Device,
        swapchain_extent: vulkanalia::vk::Extent2D,
        swapchain_image_count: usize,
        render_pass: vulkanalia::vk::RenderPass
    ) -> PResult<GraphicPipelineFFFOO<
        FrameUniform1,
        FrameUniform2,
        FrameUniform3,
        ObjectUniform1,
        ObjectUniform2
    >> {
        // create shader modules (compile byte code)
        let vert_shader_module = create_shader_module(&self.vertex_shader, vk_device)?;
        let frag_shader_module = create_shader_module(&self.fragment_shader, vk_device)?;

        let mut shader_stages = HashMap::with_capacity(2);
        shader_stages.insert(vulkanalia::vk::ShaderStageFlags::VERTEX, vert_shader_module);
        shader_stages.insert(vulkanalia::vk::ShaderStageFlags::FRAGMENT, frag_shader_module);

        // create the descriptor pool, to allocate descriptor sets.
        let vk_descriptor_pool = self.create_descriptor_pool(vk_device, swapchain_image_count)?;

        // create the uniforms
        let frame_1_uniform_buffer = self.frame_uniform_1_buffer_builder.build(vk_device, vk_descriptor_pool, swapchain_image_count)?;
        let frame_2_uniform_buffer = self.frame_uniform_2_buffer_builder.build(vk_device, vk_descriptor_pool, swapchain_image_count)?;
        let frame_3_uniform_buffer = self.frame_uniform_3_buffer_builder.build(vk_device, vk_descriptor_pool, swapchain_image_count)?;
        let object_1_uniform_buffer = self.object_uniform_1_buffer_builder.build(vk_device, vk_descriptor_pool, swapchain_image_count)?;
        let object_2_uniform_buffer = self.object_uniform_2_buffer_builder.build(vk_device, vk_descriptor_pool, swapchain_image_count)?;

        let layouts = vec![
            frame_1_uniform_buffer.layout(),
            frame_2_uniform_buffer.layout(),
            frame_3_uniform_buffer.layout(),
            object_1_uniform_buffer.layout(),
            object_2_uniform_buffer.layout(),
        ];
        
        // pipeline layout is where we set all our uniforms declaration
        let layout_info = vulkanalia::vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(&layouts);

        // create the pipeline layout and the pipeline.
        let pipeline_layout = unsafe { vk_device.create_pipeline_layout(&layout_info, None)? };
        
        // set the vertex input state
        let vertex_binding_description = vec![Vertex::binding_description()];
        let vertex_attribute_description = Vertex::attribute_description();
        
        GraphicPipelineFFFOO::create(
            vk_device, 
            vertex_binding_description,
            vertex_attribute_description,
            shader_stages,
            swapchain_extent,
            pipeline_layout,
            render_pass,
            vk_descriptor_pool,
            frame_1_uniform_buffer,
            frame_2_uniform_buffer,
            frame_3_uniform_buffer,
            object_1_uniform_buffer,
            object_2_uniform_buffer,
        )
    }

    
    fn create_descriptor_pool(
        &self,
        vk_device: &vulkanalia::Device,
        frame_count: usize,
    ) -> PResult<vulkanalia::vk::DescriptorPool> {
        let descriptor_set_count = (1 + 1 + 1 + 1 + 1) * frame_count;

        // for each layout type, we count how many descriptor sets we need.
        let mut ds_count_map = HashMap::with_capacity(3);

        let types = vec![
            self.frame_uniform_1_buffer_builder.descriptor_type(),
            self.frame_uniform_2_buffer_builder.descriptor_type(),
            self.frame_uniform_3_buffer_builder.descriptor_type(),
            self.object_uniform_1_buffer_builder.descriptor_type(),
            self.object_uniform_2_buffer_builder.descriptor_type(),
        ];
        
        for ds_type in types.into_iter() {
            match ds_count_map.get_mut(&ds_type) {
                Some(count) => *count += frame_count,
                None => { ds_count_map.insert(ds_type, frame_count); },
            }
        }
        
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

}

impl<
    FrameUniform1: FrameUniform + 'static,
    FrameUniform2: FrameUniform + 'static,
    FrameUniform3: FrameUniform + 'static,
    ObjectUniform1: RenderableComponent + 'static,
    ObjectUniform2: ObjectUniform + 'static,
> GraphicPipelineBuilderInterface for GraphicPipelineBuilderFFFOO<FrameUniform1, FrameUniform2, FrameUniform3, ObjectUniform1, ObjectUniform2> {
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

pub fn default_phong_pipeline() -> impl GraphicPipelineBuilderInterface {
    GraphicPipelineBuilderFFFOO::<
        CameraUniformObject,
        MainDirectionnalLight,
        MainDirectionnalLight,
        PhongMaterial,
        ModelMatrixUniformObject,
    > {
        phantom: Default::default(),
        vertex_shader: DEFAULT_VERT.iter().map(|v| *v).collect(),
        fragment_shader: DEFAULT_FRAG.iter().map(|v| *v).collect(),
        frame_uniform_1_buffer_builder: UniformBufferBuilder::<CameraUniformObject>::new(
            vulkanalia::vk::ShaderStageFlags::VERTEX,
            vulkanalia::vk::DescriptorType::UNIFORM_BUFFER
        ),
        frame_uniform_2_buffer_builder: UniformBufferBuilder::<MainDirectionnalLight>::new(
            vulkanalia::vk::ShaderStageFlags::FRAGMENT,
            vulkanalia::vk::DescriptorType::UNIFORM_BUFFER
        ),
        frame_uniform_3_buffer_builder: UniformBufferBuilder::<MainDirectionnalLight>::new(
            vulkanalia::vk::ShaderStageFlags::FRAGMENT,
            vulkanalia::vk::DescriptorType::UNIFORM_BUFFER
        ),
        object_uniform_1_buffer_builder: UniformBufferBuilder::<PhongMaterial>::new(
            vulkanalia::vk::ShaderStageFlags::FRAGMENT,
            vulkanalia::vk::DescriptorType::STORAGE_BUFFER
        ),
        object_uniform_2_buffer_builder: UniformBufferBuilder::<ModelMatrixUniformObject>::new(
            vulkanalia::vk::ShaderStageFlags::VERTEX,
            vulkanalia::vk::DescriptorType::STORAGE_BUFFER
        ),
    }
}