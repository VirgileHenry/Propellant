use crate::{engine::{window::vulkan::{swapchain_interface::SwapchainInterface, queues::QueueFamilyIndices, rendering_command_manager::RenderingCommandManager, rendering_sync::RenderingSync}, errors::PResult}, RenderingPipelineBuilder, ProppellantResources};
use self::{rendering_pipeline_builder::rendering_pipeline_builder_states::RPBSReady, graphic_render_pass::GraphicRenderpass};
use super::graphics_pipeline::GraphicsPipeline;


pub(crate) mod final_render_target;
pub(crate) mod intermediate_render_targets;
pub(crate) mod rendering_pipeline_builder;
pub(crate) mod graphic_render_pass;

pub(crate) const MAX_FRAMES_IN_FLIGHT: usize = 1;

pub struct RenderingPipeline {
    graphic_renderpass: GraphicRenderpass,
    compute_renderpasses: Vec<()>,
    swapchain: SwapchainInterface,
    command_manager: RenderingCommandManager,
    rendering_sync: RenderingSync<MAX_FRAMES_IN_FLIGHT>,
}

impl RenderingPipeline {
    pub fn create(
        mut builder: RenderingPipelineBuilder<RPBSReady>,
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

        let (graphic_renderpass, compute_renderpasses) = if builder.state().compute_pipelines.is_empty() {
            // only graphic renderpass, no compute renderpasses.
            (
                GraphicRenderpass::create_final_pass(
                    &mut builder.state_mut().graphic_pipelines,
                    vk_device,
                    &swapchain,
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
            graphic_renderpass,
            compute_renderpasses,
            swapchain,
            command_manager,
            rendering_sync,
        })
    }


    pub fn pipeline_count(&self) -> usize {
        self.graphic_renderpass.pipelines().len()
    }

    pub fn get_pipeline(&self, id: u64) -> Option<&GraphicsPipeline> {
        self.graphic_renderpass.pipelines().get(&id)
    }

    pub fn get_pipeline_mut(&mut self, id: u64) -> Option<&mut GraphicsPipeline> {
        self.graphic_renderpass.pipelines_mut().get_mut(&id)
    }

    pub fn get_pipelines(&self) -> impl Iterator<Item = (u64, &GraphicsPipeline)> {
        self.graphic_renderpass.pipelines().iter().map(|(k, v)| (*k, v))
    }

    pub fn get_pipelines_mut(&mut self) -> impl Iterator<Item = (u64, &mut GraphicsPipeline)> {
        self.graphic_renderpass.pipelines_mut().iter_mut().map(|(k, v)| (*k, v))
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
        
        // commands for graphic renderpass
        self.graphic_renderpass.register_draw_commands(
            vk_device,
            self.command_manager.command_buffer(image_index),
            self.swapchain.extent(),
            resources,
            image_index,
        )?;
        // commands for compute renderpasses
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
        self.graphic_renderpass.recreation_cleanup(vk_device);
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
        
        self.graphic_renderpass.recreate(vk_device, &self.swapchain)?;

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
        self.graphic_renderpass.destroy(vk_device);
    }
}

