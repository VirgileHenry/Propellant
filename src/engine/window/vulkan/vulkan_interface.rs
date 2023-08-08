use std::collections::HashSet;

use crate::engine::consts::{ENGINE_VERSION, PROPELLANT_DEBUG_FEATURES};
use crate::engine::errors::debug_error::DebugError;
use crate::engine::errors::loading_errors::LoadingError;
use crate::engine::errors::rendering_error::RenderingError;
use crate::engine::errors::{PropellantError, PResult};
use crate::engine::renderer::rendering_pipeline::RenderingPipeline;
use crate::engine::renderer::rendering_pipeline::rendering_pipeline_builder::RenderingPipelineBuilder;
use crate::engine::renderer::rendering_pipeline::rendering_pipeline_builder::states::RPBSReady;

use super::physical_device_prefs::PhysicalDevicePreferences;
use super::queues::QueueFamilyIndices;
use super::swapchain_support::SwapchainSupport;
use super::transfer_command_manager::TransferCommandManager;

use vulkanalia::vk::EntryV1_0;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::KhrSurfaceExtension;

/// Extensions that are required to run the propellant engine, if we are using the window and vulkan.
pub(crate) const REQUIRED_DEVICE_EXTENSIONS: &[vulkanalia::vk::ExtensionName] = &[
    vulkanalia::vk::KHR_SWAPCHAIN_EXTENSION.name,
    vulkanalia::vk::EXT_DESCRIPTOR_INDEXING_EXTENSION.name,
];

pub struct VulkanInterface {
    pub entry: vulkanalia::Entry,
    pub instance: vulkanalia::Instance,
    pub physical_device: vulkanalia::vk::PhysicalDevice,
    pub device: vulkanalia::Device,
    pub queue: vulkanalia::vk::Queue,
    pub indices: QueueFamilyIndices,
    pub surface: vulkanalia::vk::SurfaceKHR,
    pub transfer_manager: TransferCommandManager,
}

impl VulkanInterface {
    pub fn create(
        window: &winit::window::Window,
        device_prefs: &Box<dyn PhysicalDevicePreferences>,
        app_name: String,
    ) -> PResult<VulkanInterface>{
        // create the app info as a builder 
        let application_info = vulkanalia::vk::ApplicationInfo::builder()
            .application_name(app_name.as_bytes())
            .application_version(vulkanalia::vk::make_version(1, 0, 0))
            .engine_name(b"ProppelantEngine\0")
            .engine_version(vulkanalia::vk::make_version(ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2))
            .api_version(vulkanalia::vk::make_version(1, 2, 0));
        // get the required extensions from the winit window
        let extensions = vulkanalia::window::get_required_instance_extensions(&window).iter().map(|e| e.as_ptr())
            .collect::<Vec<_>>();
        // create the vulkan loader and entry
        let loader = unsafe {
            // lib loading module error is private, so we have to go with a match here
            match vulkanalia::loader::LibloadingLoader::new(vulkanalia::loader::LIBRARY) {
                Ok(lib) => lib,
                Err(e) => return Err(PropellantError::Loading(LoadingError::VulkanLibrary(e.to_string()))),
            }
        };
        let entry = unsafe {vulkanalia::Entry::new(loader)?};

        // get the validation layer
        let available_layers = unsafe {
            entry
                .enumerate_instance_layer_properties()?
                .iter()
                .map(|l| l.layer_name)
                .collect::<HashSet<_>>()
            };

        let validation_layer = vulkanalia::vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");
        let use_validation_layers = PROPELLANT_DEBUG_FEATURES;

        if use_validation_layers && !available_layers.contains(&validation_layer) {
            return Err(PropellantError::DebugError(DebugError::MissingVulkanDebugLayers));
        }

        let layers = if use_validation_layers {
            vec![validation_layer.as_ptr()]
        } else {
            Vec::with_capacity(0)
        };

        // create the vk instance info
        let info = vulkanalia::vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(&extensions)
            .enabled_layer_names(&layers);

        // create the vk instance
        let instance = unsafe {entry.create_instance(&info, None)?};
        // create the surface : interface between vulkan and winit window.
        let surface = unsafe {vulkanalia::window::create_surface(&instance, &window, &window)?};
        // pick a physical device that match our needs
        let physical_device = Self::pick_physical_device(&instance, device_prefs, surface)?;
        // get the queue indices. 
        let indices = unsafe { QueueFamilyIndices::get(&instance, physical_device, surface)? };
        // create the actual device with the info
        let (device, queue) = Self::create_logical_device(&instance, physical_device, indices)?;

        // the transfer manager is able to send data to the gpu.
        let transfer_manager = TransferCommandManager::create(&device, indices)?;
        
        Ok(VulkanInterface {
            entry,
            instance,
            physical_device, 
            device,
            queue,
            indices,
            surface,
            transfer_manager,
        })

    }

    fn pick_physical_device(
        vk_instance: &vulkanalia::Instance,
        device_prefs: &Box<dyn PhysicalDevicePreferences>,
        surface: vulkanalia::vk::SurfaceKHR,
    ) -> PResult<vulkanalia::vk::PhysicalDevice> {
        unsafe {
            // iterate over the devices, filter the one that match the needed properties, and max them by prefs.
            vk_instance.enumerate_physical_devices()?.into_iter().filter(|device| {
                let properties = vk_instance.get_physical_device_properties(*device);
                let features = vk_instance.get_physical_device_features(*device);
                match SwapchainSupport::get(vk_instance, *device, surface) {
                    Ok(support) => if !support.is_sufficient() { return false; },
                    Err(_) => return false,
                };
                device_prefs.is_device_compatible(properties, features) &&
                Self::check_physical_device_extensions(vk_instance, *device)
            }).map(|device| {
                // todo : add swapchain support prefs.
                (device, vk_instance.get_physical_device_properties(device), vk_instance.get_physical_device_features(device))
            }).max_by(|d1, d2| device_prefs.order_devices((d1.1, d1.2), (d2.1, d2.2)))
                .and_then(|d| Some(d.0))
                .ok_or(PropellantError::Rendering(RenderingError::NoFittingVulkanDevice))
        }
    }

    fn check_physical_device_extensions(
        instance: &vulkanalia::Instance,
        physical_device: vulkanalia::vk::PhysicalDevice,
    ) -> bool {
        let extensions = unsafe {
                match instance.enumerate_device_extension_properties(physical_device, None) {
                    Ok(extensions) => extensions,
                    Err(_) => return false,
                }
            }
            .iter()
            .map(|e| e.extension_name)
            .collect::<HashSet<_>>();
        REQUIRED_DEVICE_EXTENSIONS.iter().all(|e| extensions.contains(e))
    }

    fn create_logical_device(
        vk_instance: &vulkanalia::Instance,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        indices: QueueFamilyIndices,
    ) -> PResult<(vulkanalia::Device, vulkanalia::vk::Queue)> {
        let queue_priorities = &[1.0];
        let queue_info = vulkanalia::vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.index())
            .queue_priorities(queue_priorities);
        
        let features = vulkanalia::vk::PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(true)
            .shader_sampled_image_array_dynamic_indexing(true);

        // allows to use non uniform indexing in shaders, that is required for our texture lib.
        let mut vk12_device_features = vulkanalia::vk::PhysicalDeviceVulkan12Features::builder()
            .shader_sampled_image_array_non_uniform_indexing(true)
            .descriptor_indexing(true)
            .runtime_descriptor_array(true)
            .descriptor_binding_update_unused_while_pending(true)
            .descriptor_binding_partially_bound(true)
            .descriptor_binding_variable_descriptor_count(true);
    
        let queue_infos = &[queue_info];
        let extensions = REQUIRED_DEVICE_EXTENSIONS.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
        let info = vulkanalia::vk::DeviceCreateInfo::builder()
            .queue_create_infos(queue_infos)
            .enabled_features(&features)
            .push_next(&mut vk12_device_features)
            .enabled_extension_names(&extensions);

        let vk_device = unsafe {vk_instance.create_device(vk_physical_device, &info, None)?};
        let vk_queue = unsafe { vk_device.get_device_queue(indices.index(), 0) };

        Ok((vk_device, vk_queue))
    }

    /// Build a rendering pipeline builder into a rendering pipeline that can be used.
    pub fn build_pipeline_lib(
        &mut self,
        window: &winit::window::Window,
        pipeline_lib: RenderingPipelineBuilder<RPBSReady>
    ) -> PResult<RenderingPipeline> {
        pipeline_lib.build(
            &self.instance,
            &self.device,
            self.physical_device,
            window,
            self.surface,
            self.indices,
        )
    }

    /// Operates the registered memory transfers, and wait for them to be done.
    pub fn chack_and_process_memory_transfers(&mut self) -> PResult<()> {
        // process any memory transfers that are required.
        if self.transfer_manager.need_transfers() {
            self.transfer_manager.transfer(&self.device, self.queue)?;
        }
        Ok(())
    }

    pub fn recreate_surface(&mut self, window: &winit::window::Window) -> PResult<vulkanalia::vk::SurfaceKHR> {
        // destroy previous surface
        unsafe { self.instance.destroy_surface_khr(self.surface, None); }
        // create new surface
        self.surface = unsafe {vulkanalia::window::create_surface(&self.instance, &window, &window)?};
        // return the recreated surface
        Ok(self.surface)
    }

    pub fn wait_idle(&mut self) -> PResult<()> {
        unsafe { self.device.device_wait_idle()?; }
        Ok(())
    }

}

impl Drop for VulkanInterface {
    fn drop(&mut self) {
        unsafe {
            // we have to wait for any remaining work here, to avoid destroying in flight frames
            self.device.device_wait_idle().unwrap(); // todo : handle failure here ?
            self.transfer_manager.destroy(&self.device);
            self.device.destroy_device(None);
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);

        }
    }
}
