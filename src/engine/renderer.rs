use foundry::ComponentTable;

use crate::VulkanInterface;
use self::pipeline_lib::GraphicPipelineLib;
use self::pipeline_lib_builder::GraphicPipelineLibBuilder;
use super::errors::PropellantError;
use super::mesh::mesh_renderer_builder::MeshRendererBuilder;

use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::vk::DeviceV1_0;

pub(crate) mod pipeline_lib;
pub(crate) mod pipeline_lib_builder;
pub(crate) mod rendering_pipeline;
pub(crate) mod rendering_pipeline_builder;
pub(crate) mod shaders;

pub trait VulkanRenderer {
    /// Render the scene using the vulkan interface and the components.
    fn render(&mut self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable)-> Result<vulkanalia::vk::SuccessCode, PropellantError>;
    /// Register a pipeline lib to use for rendering.
    fn use_pipeline_lib(&mut self, pipeline_lib: GraphicPipelineLib, pipeline_lib_builder: GraphicPipelineLibBuilder);
    /// Called when the surface is out of date.
    fn on_swapchain_recreation(
        &mut self, 
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vj_physical_device: vulkanalia::vk::PhysicalDevice,
        extent: vulkanalia::vk::Extent2D,
        images: &[vulkanalia::vk::Image],
        render_pass: vulkanalia::vk::RenderPass
    ) -> Result<(), PropellantError>;
}

/// Default Vulkan renderer.
/// Perform basic drawing operation using the vk interface and the components.
pub struct DefaultVulkanRenderer {
    pipeline_lib: GraphicPipelineLib,
    pipeline_lib_builder: GraphicPipelineLibBuilder,
}

impl Default for DefaultVulkanRenderer {
    fn default() -> Self {
        DefaultVulkanRenderer {
            pipeline_lib: GraphicPipelineLib::empty(),
            pipeline_lib_builder: GraphicPipelineLibBuilder::default(),
        }
    }
}

impl DefaultVulkanRenderer {
    fn update_uniform_buffer(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        components: &mut ComponentTable
    ) -> Result<(), PropellantError> {
        self.pipeline_lib.get_pipelines_mut().for_each(
            |p| {
                match p.set_uniforms(vk_device, image_index, components) {
                    _ => {/* todo : hanlde ? */}
                };
            }
        );
        Ok(())
    }
}


impl VulkanRenderer for DefaultVulkanRenderer {
    fn render(&mut self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable) -> Result<vulkanalia::vk::SuccessCode, PropellantError> {
        // drain the mesh renderer builders, and rebuild them.
        match components.drain_components::<MeshRendererBuilder>() {
            Some(builders) => {
                // here, we have at least one mesh renderer builder. let's build them with our vulkan interface !
                for (entity, mesh_renderer) in builders.map(|(e, b)|
                (e, vk_interface.build_mesh_renderer(b))
            ) {
                    match mesh_renderer {
                        Ok(mr) => {components.add_component(entity, mr);},
                        Err(e) => println!("[PROPELLANT ERROR] Failed to build mesh renderer : {e:?}"),
                    }
                }
                // now, we need to rebuild the command buffers.
                vk_interface.rebuild_draw_commands(components, &self.pipeline_lib)?;
                // process memory transfers (staging buffers to high perf memory buffers)
                vk_interface.process_memory_transfers()?;
            }
            None => {
                // no mesh renderer to build, check for rebuild flags.
                // todo : look for rebuild flags in components singleton (mesh got deactivated, etc...)
            }
        }

        
        // vulkan rendering loop
        unsafe {
            // wait for the frame on this fence to finish
            vk_interface.rendering_sync.wait_for_frame_flight_fence(&vk_interface.device)?;
            // get the image index
            let image_index = vk_interface.device
                .acquire_next_image_khr(
                    *vk_interface.swapchain,
                    u64::max_value(),
                    vk_interface.rendering_sync.image_available_semaphore(),
                    vulkanalia::vk::Fence::null(),
                )?.0 as usize;
            // wait for any in flight image
            vk_interface.rendering_sync.wait_for_in_flight_image(image_index, &vk_interface.device)?;

            // update uniform buffer
            self.update_uniform_buffer(&vk_interface.device, image_index, components)?;
                
            // submit the draw command
            let wait_semaphores = &[vk_interface.rendering_sync.image_available_semaphore(),];
            let wait_stages = &[vulkanalia::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = &[vk_interface.rendering_manager.buffers()[image_index]];
            let signal_semaphores = &[vk_interface.rendering_sync.render_finished_semaphore()];
            let submit_info = vulkanalia::vk::SubmitInfo::builder()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_stages)
                .command_buffers(command_buffers)
                .signal_semaphores(signal_semaphores);
            
            
            vk_interface.rendering_sync.reset_in_flight_frame_fence(&vk_interface.device)?;
            
            vk_interface.device.queue_submit(
                vk_interface.queue,
                &[submit_info],
                vk_interface.rendering_sync.frame_in_flight_fence(),
            )?;
            
            // present the image
            let swapchains = &[*vk_interface.swapchain];
            let image_indices = &[image_index as u32];
            let present_info = vulkanalia::vk::PresentInfoKHR::builder()
                .wait_semaphores(signal_semaphores)
                .swapchains(swapchains)
                .image_indices(image_indices);
            
            let result = vk_interface.device.queue_present_khr(vk_interface.queue, &present_info)?;
            
            // adavance the frame
            vk_interface.rendering_sync.advance_frame();

            Ok(result)
        }
    }

    fn use_pipeline_lib(
        &mut self,
        pipeline_lib: GraphicPipelineLib,
        pipeline_lib_builder: GraphicPipelineLibBuilder
    ) {
        self.pipeline_lib = pipeline_lib;
        self.pipeline_lib_builder = pipeline_lib_builder;
    }

    #[allow(unused_variables)]
    fn on_swapchain_recreation(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        extent: vulkanalia::vk::Extent2D,
        images: &[vulkanalia::vk::Image],
        render_pass: vulkanalia::vk::RenderPass,
    ) -> Result<(), PropellantError> {
        // todo we need to rebuild our pipeline !
        // self.pipeline_lib = self.pipeline_lib_builder.clone().build(vk_instance, vk_device, vk_physical_device, extent, images, render_pass)?;
        Ok(())
    }
}