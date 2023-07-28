use foundry::{ComponentTable, component_iterator};

use crate::{
    InputContext,
    Transform,
    UiEventListener, UiEvent, engine::renderer::graphics_pipeline::uniform::frame_uniform::ui_resolution::UiResolution
};

pub(crate) mod ui_events;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorPosition {
    OutOfScreen,
    InScreen{
        mouse_x: f32,
        mouse_y: f32,
        screen_width: f32,
        screen_height: f32,
        ui_res: f32,
    },
}


pub struct UiEventHandlerContext {
    last_mouse_pos: CursorPosition,
}

impl UiEventHandlerContext {
    fn on_window_input(&mut self, event: &winit::event::WindowEvent, components: &mut ComponentTable) {

        let ui_res = match components.get_singleton::<UiResolution>() {
            Some(res) => res,
            None => return,
        };

        let ui_event = match UiEvent::try_from((event, ui_res)) {
            Ok(ev) => ev,
            Err(_) => return,
        };
        
        for (_entity, (transform, listener,)) in component_iterator!(components; mut Transform, mut UiEventListener) {
            if let Some(callback) = listener.listener() {
                callback.on_event(ui_event, transform);
            }
        }
    }
}

impl InputContext for UiEventHandlerContext {
    fn handle_device_input(&mut self, _device_id: winit::event::DeviceId, _input: winit::event::DeviceEvent, _components: &mut ComponentTable) { }

    fn handle_window_input(&mut self, input: &winit::event::WindowEvent, components: &mut ComponentTable) {
        self.on_window_input(input, components);
    }

    fn on_become_active(&mut self, _components: &mut foundry::ComponentTable) { }

    fn update(&mut self, _components: &mut foundry::ComponentTable, _delta: f32) { }
}

impl Default for UiEventHandlerContext {
    fn default() -> Self {
        UiEventHandlerContext {
            last_mouse_pos: CursorPosition::OutOfScreen,
        }
    }
}