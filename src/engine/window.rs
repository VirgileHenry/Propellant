use foundry::{ComponentTable, Updatable, System, AsAny};
use crate::{MeshLibrary, engine::consts::PROPELLANT_DEBUG_FEATURES};

use self::vulkan::vulkan_interface::VulkanInterface;

use super::{errors::PResult, renderer::VulkanRenderer};

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
    pub fn handle_event(&mut self, event: winit::event::WindowEvent, control_flow: &mut winit::event_loop::ControlFlow, _components: &mut ComponentTable) {
        match event {
            winit::event::WindowEvent::CloseRequested => control_flow.set_exit(),
            winit::event::WindowEvent::Resized(_) => {
                match self.vk_swapchain_recreation_request() {
                    Ok(_) => {},
                    Err(e) => println!("[PROPELLANT ERROR] Error while recreating swapchain: {:?}", e),
                };
            }
            _ => {},
        }
    }

    pub fn vk_swapchain_recreation_request(&mut self) -> PResult<()> {
        // signal the vk interface to recreate the swapchain.
        self.vk_interface.swapchain_recreation_request(&self.window)?;
        // rebuild the pipeline !
        self.renderer.on_swapchain_recreation(
            &self.vk_interface.instance,
            &self.vk_interface.device,
            self.vk_interface.physical_device,
            self.vk_interface.swapchain.extent(),
            &self.vk_interface.swapchain.images(),
            self.vk_interface.render_pass,
        )?;
        Ok(())
    }

    pub fn vk_interface(&self) -> &VulkanInterface {
        &self.vk_interface
    }

    pub fn vk_interface_mut(&mut self) -> &mut VulkanInterface {
        &mut self.vk_interface
    }

    pub fn world_clean_up(&mut self, components: &mut ComponentTable) {
        // wait any remaining work on the vulkan side
        match self.vk_interface.wait_idle() {
            Ok(_) => {},
            Err(e) => {
                if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] Error while waiting for vulkan idle before clean up: {:?}", e)
                }
            },
        };

        // clean up mesh library
        match components.remove_singleton::<MeshLibrary>() {
            Some(mut mesh_lib) => mesh_lib.destroy(&self.vk_interface.device),
            None => {},
        }

        // clean up the renderer
        self.renderer.destroy(&self.vk_interface.device);
    }

}

impl Updatable for PropellantWindow {
    fn update(&mut self, components: &mut ComponentTable, delta: f32) {
        // rendering loop : 
        // look for any mesh renderer builder to build 
        // redraw the scene
        // check for invalidation of the swapchain
        match self.renderer.render(&mut self.vk_interface, components, delta) {
            Ok(_) => {}, // todo : handle non optimal swapchain ok value
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