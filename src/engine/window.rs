use foundry::{ComponentTable, Updatable, System, AsAny};
use self::vulkan::vulkan_interface::VulkanInterface;

use super::{errors::PropellantError, renderer::VulkanRenderer};

#[derive(AsAny)]
pub struct PropellantWindow {
    vk_interface: VulkanInterface,
    window: winit::window::Window,
    renderer: Box<dyn VulkanRenderer>,
}

impl PropellantWindow {

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    /// handle window events. This does not need to be a self func, as the window threw the event.
    /// Further more, the window can be found in the comp table.
    pub fn handle_event(&self, event: winit::event::WindowEvent, control_flow: &mut winit::event_loop::ControlFlow, components: &mut ComponentTable) {
        match event {
            winit::event::WindowEvent::CloseRequested => control_flow.set_exit(),
            winit::event::WindowEvent::Resized(_) => {
                match match components.get_singleton_mut::<PropellantWindow>() {
                    Some(window) => window.vk_swapchain_recreation_request(),
                    None => return, // no window, no vulkan interface, no swapchain to recreate
                } {
                    Ok(_) => {},
                    Err(e) => println!("[PROPELLANT ERROR] Error while recreating swapchain: {:?}", e),
                };
            }
            _ => {},
        }
    }

    pub fn vk_swapchain_recreation_request(&mut self) -> Result<(), PropellantError> {
        // signal the vk interface to recreate the swapchain.
        self.vk_interface.swapchain_recreation_request(&self.window)
    }

    pub fn vk_interface(&self) -> &VulkanInterface {
        &self.vk_interface
    }

    pub fn vk_interface_mut(&mut self) -> &mut VulkanInterface {
        &mut self.vk_interface
    }

    pub fn world_clean_up(&mut self, components: &mut ComponentTable) {
        self.vk_interface.clean_up(components);
    }

}

impl Updatable for PropellantWindow {
    fn update(&mut self, components: &mut ComponentTable, _delta: f32) {
        // rendering loop : 
        // look for any mesh renderer builder to build 
        // redraw the scene
        // check for invalidation of the swapchain
        match self.renderer.render(&mut self.vk_interface, components) {
            Ok(_) => {},
            Err(e) => println!("[PROPELLANT ERROR] Error while rendering: {:?}", e),
        };
    }
}

/// Convert the propellant window into a foundry system that updates every frame.
impl Into<System> for PropellantWindow {
    fn into(self) -> System {
        System::new(Box::new(self), foundry::UpdateFrequency::PerFrame)
    }
}

pub(crate) mod window_builder;
pub(crate) mod vulkan;