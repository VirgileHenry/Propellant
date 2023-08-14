use foundry::{ComponentTable, AsAny};

use crate::{
    engine::consts::PROPELLANT_DEBUG_FEATURES,
    PropellantResources, Camera
};

use self::vulkan::vulkan_interface::VulkanInterface;

use super::{errors::PResult, renderer::VulkanRenderer};
#[cfg(feature = "ui")]
use super::renderer::graphic_pipeline::uniform::frame_uniform::ui_resolution::UiResolution;

#[derive(AsAny)]
pub struct PropellantWindow {
    vk_interface: VulkanInterface,
    window: winit::window::Window,
    #[cfg(feature = "vulkan-renderer")]
    renderer: Box<dyn VulkanRenderer>,
}

impl PropellantWindow {

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn render(&mut self, components: &mut ComponentTable) {
        #[cfg(feature = "vulkan-renderer")]
        {
            match self.renderer.render(&mut self.vk_interface, components) {
                Ok(_) => {},
                Err(e) => if cfg!(feature = "propellant-debug") {
                    println!("[PROPELLANT DEBUG] [WINDOW] Error while rendering frame: {e}");
                },
            }
        }
    }

    #[cfg(not(feature = "ui"))]
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
                        for (_, camera) in components.query1d_mut::<Camera>() {
                            if camera.is_main() {
                                camera.resize(new_size.height as f32, new_size.width as f32);
                            }
                        }
                    },
                    Err(e) => println!("{e} handling window resize event."),
                };
            }
            _ => {},
        }
    }

    #[cfg(feature = "ui")]
    /// handle window events. This does not need to be a self func, as the window threw the event.
    pub fn handle_event(&mut self, event: winit::event::WindowEvent, control_flow: &mut winit::event_loop::ControlFlow, components: &mut ComponentTable) {
        use crate::PropellantFlag;

        use super::engine_events::PropellantEventSenderExt;

        match event {
            winit::event::WindowEvent::CloseRequested => control_flow.set_exit(),
            winit::event::WindowEvent::Resized(new_size) => {
                match self.recreate_swapchain() {
                    Ok(_) => {
                        // command buffer will get invalidated.
                        match components.send_flag(PropellantFlag::RequireCommandBufferRebuild) {
                            Ok(_) => {},
                            Err(e) => println!("{e} sending command buffer rebuild flag."),
                        };
                        // resize main cameras
                        for (_, camera) in components.query1d_mut::<Camera>() {
                            if camera.is_main() {
                                camera.resize(new_size.height as f32, new_size.width as f32);
                            }
                        }
                        // resize ui resolution
                        match components.get_singleton_mut::<UiResolution>() {
                            Some(mut ui_res) => {
                                let (width, height) = self.window_inner_size();
                                ui_res.screen_width = width;
                                ui_res.screen_height = height;
                            },
                            None => {},
                        }
                    },
                    Err(e) => println!("{e} handling window resize event."),
                };
            }
            _ => {},
        }
    }

    pub fn recreate_swapchain(&mut self) -> PResult<()> {

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

    pub fn renderer(&self) -> &dyn VulkanRenderer {
        self.renderer.as_ref()
    }

    pub fn renderer_mut(&mut self) -> &mut dyn VulkanRenderer {
        self.renderer.as_mut()
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
        match components.remove_singleton::<PropellantResources>() {
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



pub(crate) mod window_builder;
pub(crate) mod vulkan;