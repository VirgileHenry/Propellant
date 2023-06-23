use std::collections::HashSet;

use crate::engine::consts::{ENGINE_VERSION, PROPELLANT_DEBUG_FEATURES};
use crate::engine::errors::debug_error::DebugError;
use crate::engine::errors::loading_errors::LoadingError;
use crate::engine::errors::rendering_error::RenderingError;
use crate::engine::errors::{PropellantError, PResult};
use crate::engine::renderer::pipeline_lib::GraphicPipelineLib;
use crate::engine::renderer::pipeline_lib::pipeline_lib_builder::GraphicPipelineLibBuilder;

use foundry::ComponentTable;

use super::rendering_command_manager::RenderingCommandManager;
use super::physical_device_prefs::PhysicalDevicePreferences;
use super::queues::QueueFamilyIndices;
use super::rendering_sync::RenderingSync;
use super::swapchain_interface::SwapchainInterface;
use super::swapchain_support::SwapchainSupport;
use super::transfer_command_manager::TransferCommandManager;

use vulkanalia::vk::EntryV1_0;
use vulkanalia::vk::InstanceV1_0;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::vk::KhrSurfaceExtension;

/// Extensions that are required to run the propellant engine, if we are using the window and vulkan.
pub(crate) const REQUIRED_DEVICE_EXTENSIONS: &[vulkanalia::vk::ExtensionName] = &[
    vulkanalia::vk::KHR_SWAPCHAIN_EXTENSION.name,
    vulkanalia::vk::EXT_DESCRIPTOR_INDEXING_EXTENSION.name,
];
pub(crate) const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct VulkanInterface {
    pub entry: vulkanalia::Entry,
    pub instance: vulkanalia::Instance,
    pub physical_device: vulkanalia::vk::PhysicalDevice,
    pub device: vulkanalia::Device,
    pub queue: vulkanalia::vk::Queue,
    pub indices: QueueFamilyIndices,
    pub surface: vulkanalia::vk::SurfaceKHR,
    pub swapchain: SwapchainInterface,
    pub render_pass: vulkanalia::vk::RenderPass,
    pub framebuffers: Vec<vulkanalia::vk::Framebuffer>,
    pub rendering_manager: RenderingCommandManager,
    pub transfer_manager: TransferCommandManager,
    pub rendering_sync: RenderingSync<MAX_FRAMES_IN_FLIGHT>,
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
        // create the swap chain 
        let swapchain = SwapchainInterface::create(&instance, window, surface, physical_device, &device, indices)?;
        let swapchain_images = unsafe { device.get_swapchain_images_khr(*swapchain)? };
        // create the render pass
        let render_pass = Self::create_render_pass(&device, swapchain.format())?;

        // create the frame buffers
        let framebuffers = Self::create_framebuffers(&device, &swapchain.image_views(), render_pass, swapchain.extent())?;

        // create the command pool and buffers
        let rendering_manager = RenderingCommandManager::create(&device, &framebuffers, indices)?;
        let transfer_manager = TransferCommandManager::create(&device, indices)?;

        let rendering_sync = RenderingSync::create(&device, &swapchain_images)?;


        Ok(VulkanInterface {
            entry,
            instance,
            physical_device, 
            device,
            queue,
            indices,
            surface,
            swapchain,
            render_pass,
            framebuffers,
            rendering_manager,
            transfer_manager,
            rendering_sync,
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

    fn create_render_pass(
        device: &vulkanalia::Device,
        swapchain_format: vulkanalia::vk::Format
    ) -> PResult<vulkanalia::vk::RenderPass> {
        // create the color attachment
        let color_attachment = vulkanalia::vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vulkanalia::vk::SampleCountFlags::_1)
            .load_op(vulkanalia::vk::AttachmentLoadOp::CLEAR)
            .store_op(vulkanalia::vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vulkanalia::vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vulkanalia::vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vulkanalia::vk::ImageLayout::UNDEFINED)
            .final_layout(vulkanalia::vk::ImageLayout::PRESENT_SRC_KHR);
        // create the color attachment reference
        let color_attachment_ref = vulkanalia::vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vulkanalia::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        // create the subpass
        let color_attachments = &[color_attachment_ref];
        let subpass = vulkanalia::vk::SubpassDescription::builder()
            .pipeline_bind_point(vulkanalia::vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments);
        // create the subpass dependency
        let dependency = vulkanalia::vk::SubpassDependency::builder()
            .src_subpass(vulkanalia::vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vulkanalia::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vulkanalia::vk::AccessFlags::empty())
            .dst_stage_mask(vulkanalia::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vulkanalia::vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
        // create the render pass
        let attachments = &[color_attachment];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let info = vulkanalia::vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies);
        
        Ok(unsafe {
            device.create_render_pass(&info, None)?
        })
    }

    fn create_framebuffers(
        device: &vulkanalia::Device,
        image_views: &Vec<vulkanalia::vk::ImageView>,
        render_pass: vulkanalia::vk::RenderPass,
        extent: vulkanalia::vk::Extent2D
    ) -> PResult<Vec<vulkanalia::vk::Framebuffer>> {
        Ok(image_views
            .iter()
            .map(|i| {
                let attachments = &[*i];
                let create_info = vulkanalia::vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);

                unsafe {
                    device.create_framebuffer(&create_info, None)
                }
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    /// Recompute the draw commands buffers with the components.
    /// If the scene graphics have changed, this must be called in order to see any changes.
    pub fn rebuild_draw_commands(&mut self, components: &ComponentTable, pipeline_lib: &mut GraphicPipelineLib) -> PResult<()> {
        self.rendering_manager.register_commands(&self.device, &self.swapchain, self.render_pass, &self.framebuffers, components, pipeline_lib)
    }

    /// Build a rendering pipeline builder into a rendering pipeline that can be used.
    pub fn build_pipeline_lib(&mut self, pipeline_lib: &GraphicPipelineLibBuilder) -> PResult<GraphicPipelineLib> {
        pipeline_lib.build(&self.device, self.swapchain.extent(), &self.swapchain.images(), self.render_pass)
    }

    /// Operates the registered memory transfers, and wait for them to be done.
    pub fn process_memory_transfers(&mut self) -> PResult<()> {
        // process any memory transfers that are required.
        if self.transfer_manager.need_transfers() {
            self.transfer_manager.transfer(&self.device, self.queue)?;
        }
        Ok(())
    }

    #[allow(unreachable_code, unused_variables)] // temp, while we make this working.
    pub fn swapchain_recreation_request(&mut self, window: &winit::window::Window) -> PResult<()> {
        // ! fixme this whole thing does not work ! still out of date khr !
        return Ok(());
        // first, wait for any remaining work
        unsafe { self.device.device_wait_idle()?; }
        // destroy all previous fields
        // framebuffers
        self.framebuffers.iter().for_each(|f| unsafe { self.device.destroy_framebuffer(*f, None); });
        // command buffers
        self.rendering_manager.free_command_buffers(&self.device);
        // render pass
        unsafe {self.device.destroy_render_pass(self.render_pass, None);}
        // now recreate everything !
        // swapchain -> this will delete the old swapchain
        self.swapchain.recreate(&self.instance, window, self.surface, self.physical_device, &self.device, self.indices)?;
        // render pass
        self.render_pass = Self::create_render_pass(&self.device, self.swapchain.format())?;
        // framebuffers
        self.framebuffers = Self::create_framebuffers(&self.device, self.swapchain.image_views(), self.render_pass, self.swapchain.extent())?;
        // command buffers
        self.rendering_manager.recreate_command_buffers(&self.device, &self.framebuffers)?;
        // resize in flight image
        self.rendering_sync.resize_images_in_flight(self.swapchain.images().len());
        Ok(())
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
            self.rendering_sync.destroy(&self.device);
            self.rendering_manager.destroy(&self.device);
            self.framebuffers
                .iter()
                .for_each(|f| self.device.destroy_framebuffer(*f, None));
            self.swapchain.destroy(&self.device);
            self.device.destroy_render_pass(self.render_pass, None);
            self.device.destroy_device(None);
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);

        }
    }
}
