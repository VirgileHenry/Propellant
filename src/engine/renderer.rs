use std::any::TypeId;
use std::collections::BTreeMap;
use std::collections::HashMap;

use foundry::ComponentTable;
use foundry::component_iterator;

use crate::MeshRenderer;
use crate::PropellantFlag;
use crate::ProppellantResources;
use crate::RequireCommandBufferRebuildFlag;
use crate::Transform;
use crate::VulkanInterface;
use self::rendering_pipeline::RenderingPipeline;
use self::rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder;
use self::rendering_pipeline::rendering_pipeline_builder::rendering_pipeline_builder_states::RPBSReady;
use super::consts::PROPELLANT_DEBUG_FEATURES;
use super::errors::PResult;
use super::errors::PropellantError;
use super::flags::RequireMemoryTransfersFlag;
use super::flags::RequireResourcesLoadingFlag;
use super::flags::RequireSceneRebuildFlag;
use super::window::vulkan::queues::QueueFamilyIndices;

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
    /// Destroy the current rendering pipeline.
    fn recreation_cleanup(&mut self, vk_device: &vulkanalia::Device);
    /// Clean up of all the vulkan resources.
    fn destroy(&mut self, vk_device: &vulkanalia::Device);
}


#[derive(Debug)]
struct SyncingState {
    /// for each flag type id, a vec of synced frames.
    /// If the flag is not in the map, the renderer is synced accross all frames for this flag.
    syncing_frames: HashMap<TypeId, (u64, Vec<bool>)>,
}

impl SyncingState {
    pub fn new() -> SyncingState {
        SyncingState {
            syncing_frames: HashMap::new(),
        }
    }

    fn check_flag<F: PropellantFlag + 'static>(
        &mut self,
        current_frame: usize,
        frame_count: usize,
        components: &mut ComponentTable
    ) -> Option<F> {
        match components.remove_singleton::<F>() {
            Some(flag) => {
                // if the flag is in the map, we need sync. insert a new sync vec and return true.
                let mut syncing_state = vec![true; frame_count];
                syncing_state[current_frame] = true;
                let _ = self.syncing_frames.insert(std::any::TypeId::of::<F>(), (flag.flag(), vec![false; frame_count]));
                Some(flag)
            },
            None => {
                // flag not in the components, check for syncing in our map.
                match self.syncing_frames.remove(&std::any::TypeId::of::<F>()) {
                    Some((flag, mut sync_vec)) => {
                        // check if we need to sync, then check if all frames are synced in which case we remove the flag from the map.
                        let result = !sync_vec[current_frame]; 
                        sync_vec[current_frame] = true;
                        if !sync_vec.iter().all(|synced| *synced) {
                            self.syncing_frames.insert(std::any::TypeId::of::<F>(), (flag, sync_vec));
                        }
                        if result {
                            Some(F::from_flag(flag))
                        } else {
                            None
                        }
                    },
                    None => None, // no sync required at all.
                }
            },
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

    /// Checks flags in the singleton components.
    fn handle_rendering_flags(
        &mut self,
        vk_interface: &mut VulkanInterface,
        components: &mut ComponentTable,
        current_frame: usize,
    ) -> PResult<()> {
        // ! Some flag handling much be done in a specific order.

        if let Some(flags) = self.syncing_state.check_flag::<RequireResourcesLoadingFlag>(current_frame, self.rendering_pipeline.swapchain_image_count(), components) {
            match components.get_singleton_mut::<ProppellantResources>() {
                Some(resource_lib) => {
                    // load meshes
                    resource_lib.load_resources(
                        flags,
                        &vk_interface.instance,
                        &vk_interface.device,
                        vk_interface.physical_device,
                        &mut vk_interface.transfer_manager,
                    )?;
                    // rebuild the resources uniforms.
                    for (_, pipeline) in self.rendering_pipeline.get_pipelines_mut() {
                        pipeline.rebuild_resources_uniforms(&vk_interface.device, resource_lib)?;
                    }
                    // ask for memory transfers
                    components.add_singleton(RequireMemoryTransfersFlag);
                },
                None => {
                    if PROPELLANT_DEBUG_FEATURES {
                        println!("[PROPELLANT DEBUG] Resources reloading flag found, but no resource lib found.");
                    }
                }
            }
        }

        if let Some(_) = self.syncing_state.check_flag::<RequireSceneRebuildFlag>(current_frame, self.rendering_pipeline.swapchain_image_count(), components) {
            self.scene_recreation(
                vk_interface,
                components,
                current_frame,
            )?;
        }

        if let Some(_) = self.syncing_state.check_flag::<RequireCommandBufferRebuildFlag>(current_frame, self.rendering_pipeline.swapchain_image_count(), components) {
            match components.get_singleton_mut::<ProppellantResources>() {
                Some(resource_lib) => {
                    self.rendering_pipeline.register_draw_commands(
                        &vk_interface.device,
                        &resource_lib,
                        current_frame
                    )?;
                },
                None => {
                    if PROPELLANT_DEBUG_FEATURES {
                        println!("[PROPELLANT DEBUG] Reregister draw commands flag found, but no resource lib found.");
                    }
                }
            }
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
        self.rendering_pipeline.get_pipelines_mut().for_each(
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
        for (_, pipeline) in self.rendering_pipeline.get_pipelines_mut() {
            pipeline.update_frame_uniform_buffers(components, image_index)?;
        }

        // upload all object uniform memory
        for (entity, (transform, mesh_renderer  )) in component_iterator!(components; mut Transform, MeshRenderer) {
            // skip static mesh renderers (no buffer updates)
            if mesh_renderer.is_static() {
                continue;
            }
            // get the pipeline, update to it's uniforms buffers
            match self.rendering_pipeline.get_pipeline_mut(mesh_renderer.pipeline_id()) {
                Some(pipeline) => {pipeline.update_uniform_buffers(mesh_renderer.instance(), transform, mesh_renderer.material(), image_index)?;},
                None => {
                    if cfg!(debug_assertions) {
                        println!("[PROPELLANT DEBUG] Pipeline id {} requested by entity {} does not exist.", mesh_renderer.pipeline_id(), entity);
                    }
                }
            };
        }

        // unmap all the uniform buffers
        self.rendering_pipeline.get_pipelines_mut().for_each(
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
        let mut instance_count: HashMap<u64, BTreeMap<u64, (usize, usize, usize)>> = HashMap::with_capacity(self.rendering_pipeline.pipeline_count());
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
            match self.rendering_pipeline.get_pipeline_mut(pipeline_id) {
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
        self.rendering_pipeline.get_pipelines_mut().for_each(
            |(_, pipeline)| match pipeline.map_all_uniform_buffers(&vk_interface.device, image_index) {
                Ok(_) => {/* all good */}
                Err(e) => {
                    if cfg!(debug_assertions) {
                        println!("{e} Failed to map uniform buffers.");
                    }
                }
            }
        );

        let swapchain_image_count = self.rendering_pipeline.swapchain_image_count();

        for (entity, (tf, mesh_renderer)) in component_iterator!(components; mut Transform, MeshRenderer) {
            if mesh_renderer.is_static() {
                match self.rendering_pipeline.get_pipeline_mut(mesh_renderer.pipeline_id()) {
                    Some(pipeline) => {
                        for i in 0..swapchain_image_count {
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
        self.rendering_pipeline.get_pipelines_mut().for_each(
            |(_, pipeline)| pipeline.unmap_all_uniform_buffers(&vk_interface.device, image_index)
        );

        // finally, recreate the command buffers
        // directly return the result
        let resources = match components.get_singleton::<ProppellantResources>() {
            Some(resources) => resources,
            None => return Err(PropellantError::NoResources),
        };
        self.rendering_pipeline.register_draw_commands(&vk_interface.device, resources, image_index)
    }
}


impl VulkanRenderer for DefaultVulkanRenderer {
    fn render(&mut self, vk_interface: &mut VulkanInterface, components: &mut ComponentTable, _delta_time: f32) -> PResult<vulkanalia::vk::SuccessCode> {

        
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
            self.handle_rendering_flags(vk_interface, components, image_index)?;

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

    fn recreation_cleanup(&mut self, vk_device: &vulkanalia::Device) {
        self.rendering_pipeline.recreation_cleanup(vk_device);
    }

    fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        self.rendering_pipeline.destroy(vk_device);
    }
}