use crate::{engine::{window::vulkan::{swapchain_interface::SwapchainInterface, queues::QueueFamilyIndices, rendering_command_manager::RenderingCommandManager, rendering_sync::RenderingSync}, errors::PResult}, RenderingPipelineBuilder, ProppellantResources};
use self::{rendering_pipeline_builder::rendering_pipeline_builder_states::RPBSReady, graphic_render_pass::GraphicRenderpass};
use super::graphics_pipeline::GraphicsPipeline;


pub(crate) mod attachments;
pub(crate) mod final_render_target;
pub(crate) mod intermediate_render_targets;
pub(crate) mod rendering_pipeline_builder;
pub(crate) mod graphic_render_pass;

pub(crate) const MAX_FRAMES_IN_FLIGHT: usize = 1;

pub struct RenderingPipeline {
    graphic_render_pass: GraphicRenderpass,
    #[allow(unused)]
    compute_render_passes: Vec<()>,
    swapchain: SwapchainInterface,
    command_manager: RenderingCommandManager,
    rendering_sync: RenderingSync<MAX_FRAMES_IN_FLIGHT>,
}

impl RenderingPipeline {
    pub fn create(
        builder: RenderingPipelineBuilder<RPBSReady>,
        vk_instance: &vulkanalia::Instance,
        window: &winit::window::Window,
        surface: vulkanalia::vk::SurfaceKHR,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        queue_indices: QueueFamilyIndices,
    ) -> PResult<RenderingPipeline> {

        let swapchain = SwapchainInterface::create(
            vk_instance,
            window,
            surface,
            vk_physical_device,
            vk_device,
            queue_indices
        )?;

        let clear_color = builder.clear_color();
        let mut builder_state: RPBSReady = builder.into();

        let (graphic_render_pass, compute_render_passes) = if builder_state.compute_pipelines.is_empty() {
            // only graphic render_pass, no compute render_passes.
            (
                GraphicRenderpass::create_final_pass(
                    &mut builder_state.graphic_pipelines,
                    builder_state.final_render_target,
                    vk_instance,
                    vk_device,
                    vk_physical_device,
                    &swapchain,
                    clear_color,
                )?,
                Vec::with_capacity(0),
            )
        } else {
            unimplemented!()
        };

        // create sync system and transfer manager
        let command_manager = RenderingCommandManager::create(vk_device, swapchain.images().len(), queue_indices)?;
        let rendering_sync = RenderingSync::create(vk_device, swapchain.images().len())?;

        Ok(RenderingPipeline {
            graphic_render_pass,
            compute_render_passes,
            swapchain,
            command_manager,
            rendering_sync,
        })
    }


    pub fn pipeline_count(&self) -> usize {
        self.graphic_render_pass.pipelines().len()
    }

    pub fn get_pipeline(&self, id: u64) -> Option<&GraphicsPipeline> {
        self.graphic_render_pass.pipelines().get(&id)
    }

    pub fn get_pipeline_mut(&mut self, id: u64) -> Option<&mut GraphicsPipeline> {
        self.graphic_render_pass.pipelines_mut().get_mut(&id)
    }

    pub fn get_pipelines(&self) -> impl Iterator<Item = (u64, &GraphicsPipeline)> {
        self.graphic_render_pass.pipelines().iter().map(|(k, v)| (*k, v))
    }

    pub fn get_pipelines_mut(&mut self) -> impl Iterator<Item = (u64, &mut GraphicsPipeline)> {
        self.graphic_render_pass.pipelines_mut().iter_mut().map(|(k, v)| (*k, v))
    }

    pub fn swapchain(&self) -> &SwapchainInterface {
        &self.swapchain
    }

    pub fn swapchain_image_count(&self) -> usize {
        self.swapchain.images().len()
    }

    pub fn register_draw_commands(
        &self,
        vk_device: &vulkanalia::Device,
        resources: &ProppellantResources,
        image_index: usize,
    ) -> PResult<()> {

        // start recording
        self.command_manager.start_recording_command_buffer(vk_device, image_index)?;
        
        // commands for graphic render_pass
        self.graphic_render_pass.register_draw_commands(
            vk_device,
            self.command_manager.command_buffer(image_index),
            self.swapchain.extent(),
            resources,
            image_index,
        )?;
        // commands for compute render_passes
        // todo 

        // end recording
        self.command_manager.end_recording_command_buffer(vk_device, image_index)?;

        Ok(())
    }

    /// Destroy all surface related objects to prepare recreation.
    pub fn recreation_cleanup(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        self.graphic_render_pass.recreation_cleanup(vk_device);
        self.swapchain.destroy(vk_device);
    }

    /// rebuild all surface related objects from a new surface and window.
    pub fn recreate(
        &mut self,
        window: &winit::window::Window,
        surface: vulkanalia::vk::SurfaceKHR,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        queue_indices: QueueFamilyIndices,
    ) -> PResult<()> {
        self.swapchain = SwapchainInterface::create(
            vk_instance,
            window,
            surface,
            vk_physical_device,
            vk_device,
            queue_indices
        )?;
        
        self.graphic_render_pass.recreate(
            vk_instance,
            vk_device,
            vk_physical_device,
            &self.swapchain
        )?;

        Ok(())
    }

    pub fn rendering_sync(&self) -> &RenderingSync<MAX_FRAMES_IN_FLIGHT> {
        &self.rendering_sync
    }

    pub fn rendering_sync_mut(&mut self) -> &mut RenderingSync<MAX_FRAMES_IN_FLIGHT> {
        &mut self.rendering_sync
    }

    pub fn command_manager(&self) -> &RenderingCommandManager {
        &self.command_manager
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        self.command_manager.destroy(vk_device);
        self.rendering_sync.destroy(vk_device);
        self.swapchain.destroy(vk_device);
        self.graphic_render_pass.destroy(vk_device);
    }
}

