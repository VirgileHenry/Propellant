

use foundry::ComponentTable;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

use crate::MeshLibrary;
use crate::engine::errors::PResult;
use crate::engine::errors::PropellantError;
use crate::engine::errors::rendering_error::RenderingError;
use crate::engine::renderer::pipeline_lib::GraphicPipelineLib;

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
    ) -> PResult<RenderingCommandManager> {
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
    ) -> PResult<()> {
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
        components: &ComponentTable,
        pipeline_lib: &GraphicPipelineLib,
    ) -> PResult<()> {

        // get the mesh lib (to draw the meshes, duh)
        let mesh_lib = match components.get_singleton::<MeshLibrary>() {
            Some(lib) => lib,
            None => return Err(PropellantError::Rendering(RenderingError::NoMeshLibrary)),
        };

        // loop through the command buffers, and register the commands
        for (image_index, command_buffer) in self.command_buffers.iter().enumerate() {
        
            let info = vulkanalia::vk::CommandBufferBeginInfo::builder();
        
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
                .framebuffer(framebuffers[image_index])
                .render_area(render_area)
                .clear_values(clear_values);

            
            unsafe { vk_device.cmd_begin_render_pass(*command_buffer, &info, vulkanalia::vk::SubpassContents::INLINE) };
            
            // for each pipeline
            for (_, pipeline) in pipeline_lib.get_pipelines() {
                pipeline.register_draw_commands(
                    vk_device,
                    image_index,
                    *command_buffer,
                    mesh_lib,
                );
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