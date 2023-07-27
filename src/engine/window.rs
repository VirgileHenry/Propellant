use foundry::{ComponentTable, Updatable, System, AsAny, component_iterator};
use crate::{
    engine::consts::PROPELLANT_DEBUG_FEATURES,
    ProppellantResources, Camera, RequireCommandBufferRebuildFlag
};

use self::vulkan::vulkan_interface::VulkanInterface;

use super::{errors::PResult, renderer::VulkanRenderer};
#[cfg(feature = "ui")]
use super::renderer::graphics_pipeline::uniform::frame_uniform::ui_resolution::UiResolution;

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
    pub fn handle_event(&mut self, event: winit::event::WindowEvent, control_flow: &mut winit::event_loop::ControlFlow, components: &mut ComponentTable) {
        match event {
            winit::event::WindowEvent::CloseRequested => control_flow.set_exit(),
            winit::event::WindowEvent::Resized(new_size) => {
                match self.handle_window_resize() {
                    Ok(_) => {
                        // command buffer will get invalidated.
                        components.add_singleton(RequireCommandBufferRebuildFlag);
                        // resize main cameras
                        for (_, camera) in component_iterator!(components; mut Camera) {
                            if camera.is_main() {
                                camera.resize(new_size.height as f32, new_size.width as f32);
                            }
                        }
                        // resize ui resolution
                        if cfg!(feature = "ui") {
                            match components.get_singleton_mut::<UiResolution>() {
                                Some(mut ui_res) => {
                                    let (width, height) = self.window_inner_size();
                                    ui_res.screen_width = width;
                                    ui_res.screen_height = height;
                                },
                                None => {},
                            }
                        }
                    },
                    Err(e) => println!("{e} handling window resize event."),
                };
            }
            _ => {},
        }
    }

    pub fn handle_window_resize(&mut self) -> PResult<()> {

        // resize vulkan surface
        self.vk_interface.wait_idle()?;
        self.renderer.recreation_cleanup(&self.vk_interface.device);
        let new_surface = self.vk_interface.recreate_surface(&self.window)?;
        self.renderer.recreate_rendering_pipeline(
            &self.window,
            new_surface,
            &self.vk_interface.instance,
            &self.vk_interface.device,
            self.vk_interface.physical_device,
            self.vk_interface.indices,
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
                    println!("{e} while waiting for vulkan idle before clean up.")
                }
            },
        };

        // clean up mesh library
        match components.remove_singleton::<ProppellantResources>() {
            Some(mut resources) => resources.destroy(&self.vk_interface.device),
            None => {},
        }

        // clean up the renderer
        self.renderer.destroy(&self.vk_interface.device);
    }

    pub fn window_inner_size(&self) -> (f32, f32) {
        let physical_size = self.window.inner_size();
        (physical_size.width as f32, physical_size.height as f32)
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
            Err(e) => println!("{e} while rendering"),
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