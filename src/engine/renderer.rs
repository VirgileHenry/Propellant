use std::collections::BTreeMap;
use std::collections::HashMap;

use foundry::ComponentTable;
use foundry::component_iterator;

use crate::MeshRenderer;
use crate::ProppellantResources;
use crate::Transform;
use crate::VulkanInterface;
use self::rendering_pipeline::RenderingPipeline;
use self::rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder;
use self::rendering_pipeline::rendering_pipeline_builder::rendering_pipeline_builder_states::RenderingPipelineBuilderStateReady;
use super::errors::PResult;
use super::flags::RequireMemoryTransfersFlag;
use super::flags::RequireResourcesLoadingFlag;
use super::flags::RequireSceneRebuildFlag;

use vulkanalia::vk::Handle;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::vk::DeviceV1_0;

pub(crate) mod rendering_pipeline;
pub(crate) mod graphics_pipeline;
pub(crate) mod shaders;
pub(crate) mod renderer_builder;

pub trait VulkanRenderer {
    /// Render the scene using the vulkan interface and the components.
    fn render(&mut self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable, delta_time: f32)-> PResult<vulkanalia::vk::SuccessCode>;
    /// Register a pipeline lib to use for rendering.
    fn use_pipeline_lib(&mut self, pipeline_lib: RenderingPipeline, pipeline_lib_builder: RenderingPipelineBuilder<RenderingPipelineBuilderStateReady>);
    /// Called when the surface is out of date.
    fn on_swapchain_recreation(
        &mut self, 
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vj_physical_device: vulkanalia::vk::PhysicalDevice,
        extent: vulkanalia::vk::Extent2D,
        images: &[vulkanalia::vk::Image],
        render_pass: vulkanalia::vk::RenderPass
    ) -> PResult<()>;
    /// Clean up of all the vulkan resources.
    fn destroy(&mut self, vk_device: &vulkanalia::Device);
}


#[derive(Debug)]
enum SyncingState {
    /// All good
    Sane,
    /// we are syncing, with the list of unsynced frames.
    Syncing(Vec<bool>)
}

/// Default Vulkan renderer.
/// Perform basic drawing operation using the vk interface and the components.
pub struct DefaultVulkanRenderer {
    pipeline_lib: RenderingPipeline,
    pipeline_lib_builder: RenderingPipelineBuilder<RenderingPipelineBuilderStateReady>,
    syncing_state: SyncingState,
}

impl DefaultVulkanRenderer {
    pub fn new(vk_interface: &mut VulkanInterface, rendering_pipeline_builder: RenderingPipelineBuilder<RenderingPipelineBuilderStateReady>) -> PResult<DefaultVulkanRenderer> {
        let pipeline_lib = vk_interface.build_pipeline_lib(&rendering_pipeline_builder)?;
        Ok(DefaultVulkanRenderer {
            pipeline_lib,
            pipeline_lib_builder: rendering_pipeline_builder,
            syncing_state: SyncingState::Sane,
        })
    }

    /// Checks flags in the singleton components.
    fn handle_rendering_flags(
        &mut self,
        vk_interface: &mut VulkanInterface,
        components: &mut ComponentTable,
        image_index: usize,
    ) -> PResult<()> {
        // it is important that some flag are ordered.
        // for example, first build the meshes, then the scene.
        match (components.remove_singleton::<RequireResourcesLoadingFlag>(), components.get_singleton_mut::<ProppellantResources>()) {
            (Some(flags), Some(resource_lib)) => {
                // load meshes
                resource_lib.load_resources(
                    flags,
                    &vk_interface.instance,
                    &vk_interface.device,
                    vk_interface.physical_device,
                    &mut vk_interface.transfer_manager,
                )?;
                // rebuild the resources uniforms.
                for (_, pipeline) in self.pipeline_lib.get_pipelines_mut() {
                    pipeline.rebuild_resources_uniforms(&vk_interface.device, resource_lib)?;
                }
                // ask for memory transfers
                // this should be the last thing we do, so the compiler see we stop using the resource lib and we can insert comps
                components.add_singleton(RequireMemoryTransfersFlag);
            }
            _ => {}
        }
        // look for rebuild flags
        if let Some(_) = components.remove_singleton::<RequireSceneRebuildFlag>() {
            // rebuild the scene
            self.scene_recreation(
                vk_interface,
                components,
                image_index,
            )?;
            // set the syncing state : we will update the data for the current frame, but not the one for in flight frames.
            let mut unsynced_frames = vec![false; vk_interface.swapchain.images().len()];
            unsynced_frames[image_index] = true;
            self.syncing_state = SyncingState::Syncing(unsynced_frames);
        }
        // no rebuild flags, check if there is syncing to do !
        if match &mut self.syncing_state {
            SyncingState::Sane => false,
            SyncingState::Syncing(synced_frames) => {
                let result = !synced_frames[image_index];
                synced_frames[image_index] = true;
                if synced_frames.iter().all(|b| *b) {
                    self.syncing_state = SyncingState::Sane;
                }
                result
            }
        } {
            self.scene_recreation(
                vk_interface,
                components,
                image_index,
            )?;
        }

        // look for memory transfer flags
        if let Some(_) = components.remove_singleton::<RequireMemoryTransfersFlag>() {
            vk_interface.process_memory_transfers()?;
        }

        Ok(())
    }

    fn update_uniform_buffer(
        &mut self,
        vk_device: &vulkanalia::Device,
        image_index: usize,
        components: &mut ComponentTable,
    ) -> PResult<()> {

        // map all the uniform buffers
        self.pipeline_lib.get_pipelines_mut().for_each(
            |(_, pipeline)| match pipeline.map_all_uniform_buffers(vk_device, image_index) {
                Ok(_) => {/* all good */}
                Err(e) => {
                    if cfg!(debug_assertions) {
                        println!("{e}");
                    }
                }
            }
        );

        // upload all per frame objects memory
        for (_, pipeline) in self.pipeline_lib.get_pipelines_mut() {
            pipeline.update_frame_uniform_buffers(components, image_index)?;
        }

        // upload all object uniform memory
        for (entity, (transform, mesh_renderer  )) in component_iterator!(components; mut Transform, MeshRenderer) {
            // skip static mesh renderers (no buffer updates)
            if mesh_renderer.is_static() {
                continue;
            }
            // get the pipeline, update to it's uniforms buffers
            match self.pipeline_lib.get_pipeline_mut(mesh_renderer.pipeline_id()) {
                Some(pipeline) => {pipeline.update_uniform_buffers(mesh_renderer.instance(), transform, mesh_renderer.material(), image_index)?;},
                None => {
                    if cfg!(debug_assertions) {
                        println!("[PROPELLANT DEBUG] Pipeline id {} requested by entity {} does not exist.", mesh_renderer.pipeline_id(), entity);
                    }
                }
            };
        }

        // unmap all the uniform buffers
        self.pipeline_lib.get_pipelines_mut().for_each(
            |(_, pipeline)| pipeline.unmap_all_uniform_buffers(vk_device, image_index)
        );

        Ok(())
    }

    fn scene_recreation(
        &mut self,
        vk_interface: &mut VulkanInterface,
        components: &ComponentTable,
        image_index: usize,
    ) -> PResult<()> {
        // for each mesh in each pipeline, count the number of instances
        // hashmap : pipeline_id -> mesh_id -> (instance_count, mesh_offset, instance counter)
        let mut instance_count: HashMap<u64, BTreeMap<u64, (usize, usize, usize)>> = HashMap::with_capacity(self.pipeline_lib.pipeline_count());
        for (_, (_, mesh_renderer)) in component_iterator!(components; mut Transform, MeshRenderer) {
            match instance_count.get_mut(&mesh_renderer.pipeline_id()) {
                Some(meshes) => match meshes.get_mut(&mesh_renderer.mesh_id()) {
                    Some(count) => count.0 += 1,
                    None => {meshes.insert(mesh_renderer.mesh_id(), (1, 0, 0));},
                }
                None => {
                    let mut meshes = BTreeMap::new();
                    meshes.insert(mesh_renderer.mesh_id(), (1, 0, 0));
                    instance_count.insert(mesh_renderer.pipeline_id(), meshes);
                },
            };
        }

        // compute the mesh offset
        // the mesh are sorted by id, and in the uniform buffers we need objects with similare meshes to be continuous
        // so, for each mesh in ascending order, the instance count is the instance of this mesh + number of object in mesh with smaller id
        for (_, meshes) in instance_count.iter_mut() {
            // for each pipeline
            let mut offset = 0;
            // for each mesh type, 
            for (_, (count, mesh_offset, _)) in meshes.iter_mut() {
                // set that mesh offset to current offset
                *mesh_offset = offset;
                // increase current offset
                offset += *count;
            }
        }

        // now, we can iterate one second time over all objects to set their new instance id.
        // withing a pipeline, objects with same meshes will be continuous, and can be drawn in one draw call.
        for (_, (mesh_renderer, _)) in component_iterator!(components; mut MeshRenderer, Transform) {
            instance_count.get_mut(&mesh_renderer.pipeline_id()).and_then(
                |mesh_offsets| mesh_offsets.get_mut(&mesh_renderer.mesh_id()).and_then(
                    |instance_id| {
                        mesh_renderer.set_instance(instance_id.1 + instance_id.2);
                        instance_id.2 += 1;
                        Some(())
                    }
                ) 
            );
        }

        // ask each pipeline to rebuild, providing the hashmap.
        instance_count.into_iter().for_each(|(pipeline_id, map, )| {
            match self.pipeline_lib.get_pipeline_mut(pipeline_id) {
                Some(pipeline) => match pipeline.resize_uniforms_buffers(
                        map,
                        image_index,
                        &vk_interface.instance,
                        &vk_interface.device,
                        vk_interface.physical_device,
                    ) {
                    Ok(_) => {/* all good */},
                    Err(e) => {
                        if cfg!(debug_assertions) {
                            println!("{e} Failed to rebuild pipeline {pipeline_id}");
                        }
                    }
                },
                None => {
                    if cfg!(debug_assertions) {
                        println!("[PROPELLANT DEBUG] Pipeline id {} does not exist.", pipeline_id);
                    }
                }
            };
        });

        // finally, update all uniform buffers for static objects into the buffers, as they may have moved.
        // map all the uniform buffers
        self.pipeline_lib.get_pipelines_mut().for_each(
            |(_, pipeline)| match pipeline.map_all_uniform_buffers(&vk_interface.device, image_index) {
                Ok(_) => {/* all good */}
                Err(e) => {
                    if cfg!(debug_assertions) {
                        println!("{e} Failed to map uniform buffers.");
                    }
                }
            }
        );

        for (entity, (tf, mesh_renderer)) in component_iterator!(components; mut Transform, MeshRenderer) {
            if mesh_renderer.is_static() {
                match self.pipeline_lib.get_pipeline_mut(mesh_renderer.pipeline_id()) {
                    Some(pipeline) => {
                        for i in 0..vk_interface.swapchain.images().len() {
                            pipeline.update_uniform_buffers(mesh_renderer.instance(), tf, mesh_renderer.material(), i).unwrap();
                        }
                    }
                    None => {
                        if cfg!(debug_assertions) {
                            println!("[PROPELLANT DEBUG] Pipeline id {} requested by entity {} does not exist.", mesh_renderer.pipeline_id(), entity);
                        }
                    }
                };
            }
        }

        // unmap all the uniform buffers
        self.pipeline_lib.get_pipelines_mut().for_each(
            |(_, pipeline)| pipeline.unmap_all_uniform_buffers(&vk_interface.device, image_index)
        );

        // finally, recreate the command buffers
        // directly return the result
        vk_interface.rebuild_frame_draw_commands(components, &mut self.pipeline_lib, image_index)
    }
}


impl VulkanRenderer for DefaultVulkanRenderer {
    fn render(&mut self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable, _delta_time: f32) -> PResult<vulkanalia::vk::SuccessCode> {

        
        // vulkan rendering loop
        unsafe {
            // wait for the frame on this fence to finish.
            // if we have less than MAX_FRAMES_IN_FLIGHT frames in flight, this will do nothing.
            // otherwise, this will wait for the oldest frame to finish.
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

            // look for flags
            self.handle_rendering_flags(vk_interface, components, image_index)?;

            // update uniform buffer
            self.update_uniform_buffer(&vk_interface.device, image_index, components)?;

            // create the draw command
            let wait_semaphores = &[vk_interface.rendering_sync.image_available_semaphore(),];
            let wait_stages = &[vulkanalia::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let command_buffers = &[vk_interface.rendering_manager.command_buffer(image_index)];
            let signal_semaphores = &[vk_interface.rendering_sync.render_finished_semaphore()];
            let submit_info = vulkanalia::vk::SubmitInfo::builder()
                .wait_semaphores(wait_semaphores)
                .wait_dst_stage_mask(wait_stages)
                .command_buffers(command_buffers)
                .signal_semaphores(signal_semaphores);
            
            // reset the fence for this frame
            vk_interface.rendering_sync.reset_in_flight_frame_fence(&vk_interface.device)?;
            
            // submit our draw command
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
        pipeline_lib: RenderingPipeline,
        pipeline_lib_builder: RenderingPipelineBuilder<RenderingPipelineBuilderStateReady>
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
    ) -> PResult<()> {
        // todo we need to rebuild our pipeline !
        // self.pipeline_lib = self.pipeline_lib_builder.clone().build(vk_instance, vk_device, vk_physical_device, extent, images, render_pass)?;
        Ok(())
    }

    fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.pipeline_lib.destroy(vk_device);
    }
}