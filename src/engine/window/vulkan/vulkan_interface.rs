use std::collections::HashSet;

use crate::engine::consts::ENGINE_VERSION;
use crate::engine::errors::rendering_error::RenderingError;
use crate::engine::errors::{PropellantError, PResult};
use crate::engine::mesh::mesh_renderer::MeshRenderer;
use crate::engine::mesh::mesh_renderer_builder::MeshRendererBuilder;
use crate::engine::renderer::pipeline_lib::GraphicPipelineLib;
use crate::engine::renderer::pipeline_lib_builder::GraphicPipelineLibBuilder;


use foundry::ComponentTable;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::window as vk_window;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::KhrSwapchainExtension;

use super::rendering_command_manager::RenderingCommandManager;
use super::physical_device_prefs::PhysicalDevicePreferences;
use super::queues::QueueFamilyIndices;
use super::rendering_sync::RenderingSync;
use super::swapchain_interface::SwapchainInterface;
use super::swapchain_support::SwapchainSupport;
use super::transfer_command_manager::TransferCommandManager;

/// Extensions that are required to run the propellant engine, if we are using the window and vulkan.
pub(crate) const REQUIRED_DEVICE_EXTENSIONS: &[vk::ExtensionName] = &[vk::KHR_SWAPCHAIN_EXTENSION.name];
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
        app_name: String
    ,) -> PResult<VulkanInterface>{
        // create the app info as a builder 
        let application_info = vk::ApplicationInfo::builder()
            .application_name(app_name.as_bytes())
            .application_version(vk::make_version(1, 0, 0))
            .engine_name(b"ProppelantEngine\0")
            .engine_version(vk::make_version(ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2))
            .api_version(vk::make_version(1, 0, 0));
        // get the required extensions from the winit window
        let extensions = vk_window::get_required_instance_extensions(&window).iter().map(|e| e.as_ptr())
            .collect::<Vec<_>>();
        // create the vulkan loader and entry
        let loader = unsafe {
            // lib loading module error is private, so we have to go with a match here
            match LibloadingLoader::new(LIBRARY) {
                Ok(lib) => lib,
                Err(e) => return Err(PropellantError::LibLoading(e.to_string())),
            }
        };
        let entry = unsafe {Entry::new(loader)?};
        // create the vk instance info
        let info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(&extensions);
        // create the vk instance
        // todo : add validation layers
        let instance = unsafe {entry.create_instance(&info, None)?};
        // create the surface : interface between vulkan and winit window.
        let surface = unsafe {vk_window::create_surface(&instance, &window, &window)?};
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
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(indices.index())
            .queue_priorities(queue_priorities);

        let features = vk::PhysicalDeviceFeatures::builder();
    
        let queue_infos = &[queue_info];
        let extensions = REQUIRED_DEVICE_EXTENSIONS.iter().map(|e| e.as_ptr()).collect::<Vec<_>>();
        let info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(queue_infos)
            .enabled_features(&features)
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
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vk::SampleCountFlags::_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        // create the color attachment reference
        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        // create the subpass
        let color_attachments = &[color_attachment_ref];
        let subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments);
        // create the subpass dependency
        let dependency = vulkanalia::vk::SubpassDependency::builder()
            .src_subpass(vulkanalia::vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
        // create the render pass
        let attachments = &[color_attachment];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let info = vk::RenderPassCreateInfo::builder()
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
                let create_info = vk::FramebufferCreateInfo::builder()
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

    /// Build the given mesh renderer builder into a mesh renderer.
    /// This allocates ressources for the buffers in memory, and register command for transfer operations.
    /// Therefore, the mutable borrow. 
    pub fn build_mesh_renderer(&mut self, builder: MeshRendererBuilder) -> PResult<MeshRenderer> {
        Ok(builder.build(&self.instance, &self.device, self.physical_device, &mut self.transfer_manager)?)
    }

    /// Recompute the draw commands buffers with the components.
    /// If the scene graphics have changed, this must be called in order to see any changes.
    pub fn rebuild_draw_commands(&mut self, components: &mut ComponentTable, pipeline_lib: &GraphicPipelineLib) -> PResult<()> {
        self.rendering_manager.register_commands(&self.device, &self.swapchain, self.render_pass, &self.framebuffers, components, pipeline_lib)
    }

    /// Build a rendering pipeline builder into a rendering pipeline that can be used.
    pub fn build_pipeline_lib(&mut self, pipeline_lib: &GraphicPipelineLibBuilder) -> PResult<GraphicPipelineLib> {
        pipeline_lib.build(&self.instance, &self.device, self.physical_device, self.swapchain.extent(), &self.swapchain.images(), self.render_pass)
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

    /// Destroys all the ressources allocated by vulkan through the world.
    pub fn clean_up(&mut self, components: &mut ComponentTable) {
        match components.drain_components::<MeshRenderer>() {
            Some(renderers) => for (_, mut renderer) in renderers {
                renderer.destroy(&self.device);
            }
            None => {}, // no mesh renderers to destroy
        }
    }
}

impl Drop for VulkanInterface {
    fn drop(&mut self) {
        unsafe {
            // we have to wait for any remaining work here, to avoid destroying in flight frames.
            self.device.device_wait_idle().unwrap(); // todo : handle this ? 
            self.rendering_sync.destroy(&self.device);
            self.rendering_manager.destroy(&self.device);
            self.framebuffers
                .iter()
                .for_each(|f| self.device.destroy_framebuffer(*f, None));
            self.swapchain.destroy(&self.device);
            self.device.destroy_render_pass(self.render_pass, None);
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
