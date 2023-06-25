use crate::engine::consts::ENGINE_VERSION;
use crate::engine::engine_events::PropellantEvent;
use crate::engine::errors::PResult;
use crate::engine::renderer::renderer_builder::VulkanRendererBuilder;
use crate::engine::renderer::renderer_builder::default_vulkan_renderer_builder::DefaultVulkanRendererBuilder;
use super::PropellantWindow;
use super::vulkan::physical_device_prefs::{PhysicalDevicePreferences, DefaultPhysicalDevicePreferences};
use super::vulkan::vulkan_interface::VulkanInterface;



/// Contains build info for the propellant window. 
/// Wraps the vulkan build info as well.
pub struct PropellantWindowBuilder {
    app_name: String,
    device_prefs: Box<dyn PhysicalDevicePreferences>,
    renderer: Box<dyn VulkanRendererBuilder>,
    inner_size: (usize, usize),
}

impl PropellantWindowBuilder {
    pub fn build(self, event_loop: &winit::event_loop::EventLoop<PropellantEvent>) -> PResult<PropellantWindow> {
        // create the window from the event loop
        let window = winit::window::WindowBuilder::new()
            .with_title(&self.app_name)
            .with_inner_size(winit::dpi::LogicalSize::new(self.inner_size.0 as u32, self.inner_size.1 as u32))
            .build(event_loop).unwrap();
        // name of the app
        let mut vk_interface: VulkanInterface = VulkanInterface::create(&window, &self.device_prefs, self.app_name)?;
        let renderer = self.renderer.build(&mut vk_interface)?;

        Ok(PropellantWindow {
            vk_interface,
            window,
            renderer,
        })
    }

    pub fn with_title(self, app_name: String) -> PropellantWindowBuilder {
        PropellantWindowBuilder { app_name, ..self }
    }

    pub fn with_device_prefs(self, device_prefs: Box<dyn PhysicalDevicePreferences>) -> PropellantWindowBuilder {
        PropellantWindowBuilder { device_prefs, ..self }
    }

    pub fn with_renderer(self, renderer: Box<dyn VulkanRendererBuilder>) -> PropellantWindowBuilder {
        PropellantWindowBuilder { renderer, ..self }
    }

}

impl Default for PropellantWindowBuilder {
    fn default() -> Self {
        PropellantWindowBuilder {
            app_name: format!("Propellant Engine V{}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            device_prefs: Box::new(DefaultPhysicalDevicePreferences),
            renderer: DefaultVulkanRendererBuilder::default(),
            inner_size: (800, 450),
        }
    }
}

