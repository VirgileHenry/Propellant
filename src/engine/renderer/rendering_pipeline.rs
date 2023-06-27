use crate::{engine::{window::vulkan::{swapchain_interface::SwapchainInterface, queues::QueueFamilyIndices, rendering_command_manager::RenderingCommandManager, rendering_sync::RenderingSync}, errors::PResult}, RenderingPipelineBuilder, ProppellantResources};
use self::{rendering_pipeline_builder::rendering_pipeline_builder_states::RenderingPipelineBuilderStateReady, rendering_pipeline_pass::RenderingPipelinePass};
use super::graphics_pipeline::GraphicsPipeline;


pub(crate) mod rendering_pipeline_builder;
pub(crate) mod intermediate_render_targets;
pub(crate) mod rendering_pipeline_pass;

pub(crate) const MAX_FRAMES_IN_FLIGHT: usize = 1;

pub struct RenderingPipeline {
    render_passes: Vec<RenderingPipelinePass>,
    swapchain: SwapchainInterface,
    command_manager: RenderingCommandManager,
    rendering_sync: RenderingSync<MAX_FRAMES_IN_FLIGHT>,
}

impl RenderingPipeline {
    pub fn create(
        builder: RenderingPipelineBuilder<RenderingPipelineBuilderStateReady>,
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

        let (
            transition_layers,
            mut last_layer
        ) = builder.layers();

        let render_passes = transition_layers.map(|(mut layer, target)| {
            assert!(false, "TODO: implement transition layers");
            RenderingPipelinePass::create_transition_pass(
                layer.pipelines_mut(),
                &target,
                vk_instance,
                vk_device,
                vk_physical_device,
                &swapchain,
            )
        }).chain(
            std::iter::once(
                RenderingPipelinePass::create_final_pass(
                    last_layer.pipelines_mut(),
                    vk_instance,
                    vk_device,
                    vk_physical_device,
                    &swapchain,
                )
            )
        ).collect::<PResult<Vec<_>>>()?;

        // create sync system and transfer manager
        let command_manager = RenderingCommandManager::create(vk_device, swapchain.images().len(), queue_indices)?;
        let rendering_sync = RenderingSync::create(vk_device, swapchain.images().len())?;

        Ok(RenderingPipeline {
            render_passes,
            swapchain,
            command_manager,
            rendering_sync,
        })
    }


    pub fn pipeline_count(&self) -> usize {
        self.render_passes.iter().map(|renderpass| renderpass.pipelines().len()).sum()
    }

    pub fn get_pipeline(&self, id: u64) -> Option<&GraphicsPipeline> {
        self.render_passes.iter().find(|renderpass| {
            renderpass.pipelines().contains_key(&id)
        }).map(|renderpass| {
            renderpass.pipelines().get(&id).unwrap() // we can unwrap safely, as we found the key.
        })
    }

    pub fn get_pipeline_mut(&mut self, id: u64) -> Option<&mut GraphicsPipeline> {
        self.render_passes.iter_mut().find(|renderpass| {
            renderpass.pipelines().contains_key(&id)
        }).map(|renderpass| {
            renderpass.pipelines_mut().get_mut(&id).unwrap() // we can unwrap safely, as we found the key.
        })
    }

    pub fn get_pipelines(&self) -> impl Iterator<Item = (u64, &GraphicsPipeline)> {
        self.render_passes.iter().flat_map(|renderpass| renderpass.pipelines().iter().map(|(k, v)| (*k, v)))
    }

    pub fn get_pipelines_mut(&mut self) -> impl Iterator<Item = (u64, &mut GraphicsPipeline)> {
        self.render_passes.iter_mut().flat_map(|renderpass| renderpass.pipelines_mut().iter_mut().map(|(k, v)| (*k, v)))
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
        
        // commands for each render pass
        for renderpass in self.render_passes.iter() {
            renderpass.register_draw_commands(
                vk_device,
                self.command_manager.command_buffer(image_index),
                self.swapchain.extent(),
                resources,
                image_index,
            )?;
        }

        // end recording
        self.command_manager.end_recording_command_buffer(vk_device, image_index)?;

        Ok(())
    }

    /// Destroy all surface related objects to prepare recreation.
    pub fn prepare_recreation(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        self.render_passes.iter_mut().for_each(|renderpass| renderpass.prepare_recreation(vk_device));
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
        
        for renderpass in self.render_passes.iter_mut() {
            renderpass.recreate(vk_device, &self.swapchain)?;
        }

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
        for renderpass in self.render_passes.iter_mut() {
            renderpass.destroy(vk_device);
        }
    }
}

