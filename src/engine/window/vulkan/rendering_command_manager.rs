
use std::collections::HashMap;

use foundry::ComponentTable;
use foundry::component_iterator;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

use crate::MeshRenderer;
use crate::engine::errors::PropellantError;
use crate::engine::renderer::pipeline_lib::GraphicPipelineLib;
use crate::Transform;

pub struct RenderingCommandManager {
    command_pool: vulkanalia::vk::CommandPool,
    command_buffers: Vec<vulkanalia::vk::CommandBuffer>,
}

impl RenderingCommandManager {
    /// Creates a new command pool and buffers.
    pub fn create(
        vk_device: &vulkanalia::Device,
        framebuffers: &Vec<vulkanalia::vk::Framebuffer>,
        indices: super::queues::QueueFamilyIndices,
    ) -> Result<RenderingCommandManager,PropellantError> {
        // create the frame buffers
        let info = vulkanalia::vk::CommandPoolCreateInfo::builder()
            .queue_family_index(indices.index())
            .flags(vulkanalia::vk::CommandPoolCreateFlags::empty());

        let command_pool = unsafe {vk_device.create_command_pool(&info, None)?};

        let allocate_info = vulkanalia::vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vulkanalia::vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(framebuffers.len() as u32);
        
        let command_buffers = unsafe{ vk_device.allocate_command_buffers(&allocate_info)? };

        Ok(RenderingCommandManager {
            command_pool,
            command_buffers,
        })
    }

    /// Recreate the command buffers. They need to be freed before.
    pub fn recreate_command_buffers(
        &mut self,
        vk_device: &vulkanalia::Device,
        framebuffers: &Vec<vulkanalia::vk::Framebuffer>,
    ) -> Result<(), PropellantError> {
        // create the frame buffers
        let allocate_info = vulkanalia::vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(vulkanalia::vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(framebuffers.len() as u32);
        
        let command_buffers = unsafe{ vk_device.allocate_command_buffers(&allocate_info)? };

        self.command_buffers = command_buffers;

        Ok(())
    }


    pub fn buffers(&self) -> &Vec<vulkanalia::vk::CommandBuffer> {
        &self.command_buffers
    }

    pub fn register_commands(
        &mut self,
        vk_device: &vulkanalia::Device,
        swapchain: &super::swapchain_interface::SwapchainInterface,
        render_pass: vulkanalia::vk::RenderPass,
        framebuffers: &Vec<vulkanalia::vk::Framebuffer>,
        components: &mut ComponentTable,
        pipeline_lib: &GraphicPipelineLib,
    ) -> Result<(), PropellantError> {
        // loop through the command buffers, and register the commands
        for (i, command_buffer) in self.command_buffers.iter().enumerate() {
            let inheritance = vulkanalia::vk::CommandBufferInheritanceInfo::builder();
        
            let info = vulkanalia::vk::CommandBufferBeginInfo::builder()
                .flags(vulkanalia::vk::CommandBufferUsageFlags::empty()) // Optional.
                .inheritance_info(&inheritance);             // Optional.
        
            unsafe { vk_device.begin_command_buffer(*command_buffer, &info)? };
            let render_area = vulkanalia::vk::Rect2D::builder()
                .offset(vulkanalia::vk::Offset2D::default())
                .extent(swapchain.extent());
    
            let color_clear_value = vulkanalia::vk::ClearValue {
                color: vulkanalia::vk::ClearColorValue {
                    float32: [0., 0., 0., 1.0],
                },
            };
    
            let clear_values = &[color_clear_value];
            let info = vulkanalia::vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass)
                .framebuffer(framebuffers[i])
                .render_area(render_area)
                .clear_values(clear_values);

            // create a map of the mesh renderers, regrouped by pipeline.
            let mut rendering_map: HashMap<u64, Vec<(&mut Transform, &MeshRenderer)>> = HashMap::new();

            // todo : does not need to be mutable, but there is a buf in the foundry.
            for render_data in component_iterator!(components; mut Transform, MeshRenderer) {
                match rendering_map.get_mut(&render_data.1.pipeline_id()) {
                    Some(rendering_list) => {
                        rendering_list.push(render_data);
                    },
                    None => {
                        rendering_map.insert(render_data.1.pipeline_id(), vec![render_data]);
                    }
                }
            }
            
            unsafe { vk_device.cmd_begin_render_pass(*command_buffer, &info, vulkanalia::vk::SubpassContents::INLINE) };
            
            for (pipeline_id, mesh_renderers) in rendering_map.into_iter() {
                // get the pipeline
                let pipeline = match pipeline_lib.get_pipeline(pipeline_id) {
                    Some(pipeline) => pipeline,
                    None => {
                        println!("[PROPELLANT ERROR] Pipeline not found for id {}", pipeline_id);
                        continue;
                    }
                };
                // bind the pipeline
                unsafe { vk_device.cmd_bind_pipeline(*command_buffer, vulkanalia::vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline()) };
                // bind the descriptor sets
                // unsafe { vk_device.cmd_bind_descriptor_sets(*command_buffer, vulkanalia::vk::PipelineBindPoint::GRAPHICS, pipeline.layout(), 0, &pipeline.descriptor_sets(), &[]) };
                // bind the vertex buffers
                for (_transform, mesh_renderer) in mesh_renderers.into_iter() {
                    mesh_renderer.register_draw_commands(vk_device, *command_buffer);
                }
            }

            unsafe { vk_device.cmd_end_render_pass(*command_buffer) };
            unsafe { vk_device.end_command_buffer(*command_buffer)? };

        }

        Ok(())
    }
    
    /// Free the command buffers. This is also perfomed by the destroy method.
    pub fn free_command_buffers(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        unsafe { vk_device.free_command_buffers(self.command_pool, &self.command_buffers) };
    }

    /// Destroys the associated objects. 
    /// Note that this also free the command buffers, so no need to call this first.
    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        unsafe {
            vk_device.free_command_buffers(self.command_pool, &self.command_buffers);
            vk_device.destroy_command_pool(self.command_pool, None);
        }
    }
}