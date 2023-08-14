use std::collections::VecDeque;

use foundry::ComponentTable;

use crate::PropellantFlag;
use crate::PropellantResources;
use crate::VulkanInterface;
use self::rendering_pipeline::RenderingPipeline;
use self::rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder;
use self::rendering_pipeline::rendering_pipeline_builder::states::RPBSReady;
use super::consts::PROPELLANT_DEBUG_FEATURES;
use super::errors::PResult;
use super::errors::PropellantError;
use super::window::vulkan::queues::QueueFamilyIndices;

use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::vk::DeviceV1_0;

pub(crate) mod rendering_pipeline;
pub(crate) mod graphic_pipeline;
#[allow(unused)]
pub(crate) mod shaders;
pub(crate) mod renderer_builder;
pub(crate) mod rendering_map;

pub trait VulkanRenderer {
    /// Render the scene using the vulkan interface and the components.
    fn render(&mut self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable)-> PResult<vulkanalia::vk::SuccessCode>;
    /// Called when the surface is out of date. Does not destroy the previous pipeline, this is done via the `destroy_pipeline` method.
    fn recreate_rendering_pipeline(
        &mut self, 
        window: &winit::window::Window,
        surface: vulkanalia::vk::SurfaceKHR,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        queue_indices: QueueFamilyIndices,
    ) -> PResult<()>;
    /// The engine is sending a flag to the renderer.
    fn handle_engine_flag(&mut self, flag: PropellantFlag);
    /// Destroy the current rendering pipeline.
    fn recreation_cleanup(&mut self, vk_device: &vulkanalia::Device);
    /// Clean up of all the vulkan resources.
    fn destroy(&mut self, vk_device: &vulkanalia::Device);
}


#[derive(Debug)]
struct SyncingState {
    /// for each flag type id, a vec of synced frames.
    /// If the flag is not in the map, the renderer is synced accross all frames for this flag.
    syncing_frames: VecDeque<(PropellantFlag, Vec<bool>)>,
}

impl SyncingState {
    pub fn new() -> SyncingState {
        SyncingState {
            syncing_frames: VecDeque::new(),
        }
    }

    fn add_flag(&mut self, flag: PropellantFlag, frame_count: usize) {
        for (f, frames) in self.syncing_frames.iter_mut() {
            if std::mem::discriminant(&flag) == std::mem::discriminant(f) {
                frames.iter_mut().for_each(|frame| *frame = false);
                return;
            }
        }
        self.syncing_frames.push_back((flag, vec![false; frame_count]));
    }

    fn get_frame_flags(
        &mut self,
    ) -> Vec<PropellantFlag> {
        self.syncing_frames.iter().map(|(flag, _)| *flag).collect()
    }

    fn update_syncing_frames(&mut self, current_frame: usize) {
        for (_, frames) in self.syncing_frames.iter_mut() {
            frames[current_frame] = true;
        }
        while let Some((_, frames)) = self.syncing_frames.front() {
            if frames.iter().all(|frame| *frame) {
                self.syncing_frames.pop_front();
            } else {
                break;
            }
        }
    }
}

/// Default Vulkan renderer.
/// Perform basic drawing operation using the vk interface and the components.
pub struct DefaultVulkanRenderer {
    rendering_pipeline: RenderingPipeline,
    syncing_state: SyncingState,
}

impl DefaultVulkanRenderer {
    pub fn new(
        vk_interface: &mut VulkanInterface,
        window: &winit::window::Window,
        rendering_pipeline_builder: RenderingPipelineBuilder<RPBSReady>
    ) -> PResult<DefaultVulkanRenderer> {
        let pipeline_lib = vk_interface.build_pipeline_lib(
            window,
            rendering_pipeline_builder
        )?;
        Ok(DefaultVulkanRenderer {
            rendering_pipeline: pipeline_lib,
            syncing_state: SyncingState::new(),
        })
    }

    fn check_flag_handling(
        &mut self,
        vk_interface: &mut VulkanInterface,
        components: &ComponentTable,
        current_frame: usize,
    ) -> PResult<()> {
        for flag in self.syncing_state.get_frame_flags().into_iter() {
            self.handle_rendering_flag(vk_interface, components, flag, current_frame)?;
        }
        self.syncing_state.update_syncing_frames(current_frame);
        Ok(())
    }

    fn handle_rendering_flag(
        &mut self,
        vk_interface: &mut VulkanInterface,
        components: &ComponentTable,
        flag: PropellantFlag,
        current_frame: usize,
    ) -> PResult<()> {
        match flag {
            PropellantFlag::RequireSceneRebuild => self.scene_recreation(vk_interface, components, current_frame)?,
            PropellantFlag::RequireCommandBufferRebuild =>  match components.get_singleton::<PropellantResources>() {
                Some(resource_lib) => {
                    self.rendering_pipeline.register_draw_commands(
                        &vk_interface.device,
                        &resource_lib,
                        current_frame
                    )?;
                },
                None => {
                    if PROPELLANT_DEBUG_FEATURES {
                        println!("[PROPELLANT DEBUG] Re-register draw commands flag found, but no resource lib found.");
                    }
                }
            },
            _ => if PROPELLANT_DEBUG_FEATURES {
                println!("[PROPELLANT DEBUG] Flag {:?} found in renderer, but should not be handled by it.", flag);
            }
        }

        Ok(())
    }

    #[inline]
    fn update_uniform_buffer(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        components: &mut ComponentTable,
    ) -> PResult<()> {
        self.rendering_pipeline.update_uniform_buffers(
            vk_device,
            image_index,
            components,
        )
    }

    fn scene_recreation(
        &mut self,
        vk_interface: &mut VulkanInterface,
        components: &ComponentTable,
        image_index: usize,
    ) -> PResult<()> {        
        self.rendering_pipeline.scene_recreation(
            components
        )?;

        self.rendering_pipeline.assert_uniform_buffer_sizes(
            image_index,
            &vk_interface.instance,
            &vk_interface.device,
            vk_interface.physical_device,
        )?;

        let resources = match components.get_singleton::<PropellantResources>() {
            Some(resources) => resources,
            None => return Err(PropellantError::NoResources),
        };

        self.rendering_pipeline.register_draw_commands(&vk_interface.device, resources, image_index)
    }
}


impl VulkanRenderer for DefaultVulkanRenderer {
    fn render(&mut self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable) -> PResult<vulkanalia::vk::SuccessCode> {
        
        // vulkan rendering loop
        unsafe {
            // wait for the frame on this fence to finish.
            // if we have less than MAX_FRAMES_IN_FLIGHT frames in flight, this will do nothing.
            // otherwise, this will wait for the oldest frame to finish.
            self.rendering_pipeline.rendering_sync_mut().wait_for_frame_flight_fence(&vk_interface.device)?;
            // get the image index
            let image_index = vk_interface.device
                .acquire_next_image_khr(
                    self.rendering_pipeline.swapchain().swapchain(),
                    u64::max_value(),
                    self.rendering_pipeline.rendering_sync().image_available_semaphore(),
                    vulkanalia::vk::Fence::null(),
                )?.0 as usize;
                

                // wait for any in flight image
            self.rendering_pipeline.rendering_sync_mut().wait_for_in_flight_image(image_index, &vk_interface.device)?;

            // look for flags
            self.check_flag_handling(vk_interface, components, image_index)?;

            // look for memory transfer flags
            vk_interface.check_and_process_memory_transfers()?;

            // update uniform buffer
            self.update_uniform_buffer(&vk_interface.device, image_index, components)?;

            // create the draw command
            let wait_semaphores = &[self.rendering_pipeline.rendering_sync().image_available_semaphore(),];
            let wait_stages = &[vulkanalia::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = &[self.rendering_pipeline.command_manager().command_buffer(image_index)];
            let signal_semaphores = &[self.rendering_pipeline.rendering_sync().render_finished_semaphore()];
            let submit_info = vulkanalia::vk::SubmitInfo::builder()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_stages)
                .command_buffers(command_buffers)
                .signal_semaphores(signal_semaphores);
            
            // reset the fence for this frame
            self.rendering_pipeline.rendering_sync().reset_in_flight_frame_fence(&vk_interface.device)?;
            
            // submit our draw command
            vk_interface.device.queue_submit(
                vk_interface.queue,
                &[submit_info],
                self.rendering_pipeline.rendering_sync().frame_in_flight_fence(),
            )?;
            
            // present the image
            let swapchains = &[self.rendering_pipeline.swapchain().swapchain()];
            let image_indices = &[image_index as u32];
            let present_info = vulkanalia::vk::PresentInfoKHR::builder()
                .wait_semaphores(signal_semaphores)
                .swapchains(swapchains)
                .image_indices(image_indices);
            
            let result = vk_interface.device.queue_present_khr(vk_interface.queue, &present_info)?;
            
            // adavance the frame
            self.rendering_pipeline.rendering_sync_mut().advance_frame();

            Ok(result)
        }
    }

    fn recreate_rendering_pipeline(
        &mut self,
        window: &winit::window::Window,
        surface: vulkanalia::vk::SurfaceKHR,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        queue_indices: QueueFamilyIndices,
    ) -> PResult<()> {
        // recreate a pipeline
        self.rendering_pipeline.recreate(
            window,
            surface,
            vk_instance,
            vk_device,
            vk_physical_device,
            queue_indices,
        )?;
        Ok(())
    }

    fn handle_engine_flag(&mut self, flag: PropellantFlag) {
        self.syncing_state.add_flag(flag, self.rendering_pipeline.swapchain_image_count());
    }

    fn recreation_cleanup(&mut self, vk_device: &vulkanalia::Device) {
        self.rendering_pipeline.recreation_cleanup(vk_device);
    }

    fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.rendering_pipeline.destroy(vk_device);
    }
}