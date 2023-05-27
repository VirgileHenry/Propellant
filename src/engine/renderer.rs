use foundry::ComponentTable;

use crate::MeshRendererBuilder;
use crate::VulkanInterface;

use self::pipeline_lib::GraphicPipelineLib;

use super::errors::PropellantError;

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
    fn render(&self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable)-> Result<(), PropellantError>;
    fn use_pipeline_lib(&mut self, pipeline_lib: GraphicPipelineLib);
}

pub struct DefaultVulkanRenderer {
    pipeline_lib: GraphicPipelineLib,
}

impl Default for DefaultVulkanRenderer {
    fn default() -> Self {
        DefaultVulkanRenderer {
            pipeline_lib: GraphicPipelineLib::empty(),
        }
    }
}


impl VulkanRenderer for DefaultVulkanRenderer {
    fn render(&self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable) -> Result<(), PropellantError> {
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
            }
            None => {
                // no mesh renderer to build, check for rebuild flags.
                // todo : look for rebuild flags in components singleton (mesh got deactivated, etc...)
            }
        }
        // process memory transfers (staging buffers to high perf memory buffers)
        vk_interface.process_memory_transfers()?;
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
            
            vk_interface.device.queue_present_khr(vk_interface.queue, &present_info)?;
            
            // adavance the frame
            vk_interface.rendering_sync.advance_frame();
        }

        Ok(())

    }

    fn use_pipeline_lib(&mut self, pipeline_lib: GraphicPipelineLib) {
        self.pipeline_lib = pipeline_lib;
    }
}