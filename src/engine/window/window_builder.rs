use crate::engine::consts::ENGINE_VERSION;
use crate::engine::engine_events::PropellantEvent;
use crate::engine::errors::PropellantError;
use crate::engine::renderer::pipeline_lib_builder::GraphicPipelineLibBuilder;
use crate::engine::renderer::rendering_pipeline_builder::RenderingPipelineBuilder;
use crate::engine::renderer::{DefaultVulkanRenderer, VulkanRenderer};
use super::PropellantWindow;
use super::vulkan::physical_device_prefs::{PhysicalDevicePreferences, DefaultPhysicalDevicePreferences};
use super::vulkan::vulkan_interface::VulkanInterface;



/// Contains build info for the propellant window. 
/// Wraps the vulkan build info as well.
pub struct PropellantWindowBuilder {
    app_name: String,
    device_prefs: Box<dyn PhysicalDevicePreferences>,
    renderer: Box<dyn VulkanRenderer>,
    pipeline_lib: GraphicPipelineLibBuilder,
    inner_size: (usize, usize),
}

impl PropellantWindowBuilder {
    pub fn build(self, event_loop: &winit::event_loop::EventLoop<PropellantEvent>) -> Result<PropellantWindow, PropellantError> {
        // create the window from the event loop
        let window = winit::window::WindowBuilder::new()
            .with_title(&self.app_name)
            .with_inner_size(winit::dpi::LogicalSize::new(self.inner_size.0 as u32, self.inner_size.1 as u32))
            .build(event_loop).unwrap();
        // name of the app
        let mut vk_interface = VulkanInterface::create(&window, &self.device_prefs, self.app_name)?;
        let mut renderer = self.renderer;
        renderer.use_pipeline_lib(vk_interface.build_pipeline_lib(self.pipeline_lib.clone())?, self.pipeline_lib);
        
        Ok(PropellantWindow {
            vk_interface,
            window,
            renderer,
        })
    }

    pub fn with_app_name(mut self, app_name: String) -> PropellantWindowBuilder {
        self.app_name = app_name;
        self
    }

    pub fn with_device_prefs(mut self, device_prefs: Box<dyn PhysicalDevicePreferences>) -> PropellantWindowBuilder {
        self.device_prefs = device_prefs;
        self
    }

    pub fn with_renderer(mut self, renderer: Box<dyn VulkanRenderer>) -> PropellantWindowBuilder {
        self.renderer = renderer;
        self
    }

    pub fn with_pipeline(mut self, id: u64, pipeline: RenderingPipelineBuilder) -> PropellantWindowBuilder {
        self.pipeline_lib.register_pipeline(id, pipeline);
        self
    }
}

impl Default for PropellantWindowBuilder {
    fn default() -> Self {
        PropellantWindowBuilder {
            app_name: format!("Propellant Engine V{}.{}.{}", ENGINE_VERSION.0, ENGINE_VERSION.1, ENGINE_VERSION.2),
            device_prefs: Box::new(DefaultPhysicalDevicePreferences),
            renderer: Box::new(DefaultVulkanRenderer::default()),
            pipeline_lib: GraphicPipelineLibBuilder::default(),
            inner_size: (800, 450),
        }
    }
}

