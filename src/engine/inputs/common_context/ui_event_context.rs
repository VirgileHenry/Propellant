use foundry::ComponentTable;

use crate::{
    InputContext,
    Transform,
    UiEventListener,
    UiEvent,
};

pub(crate) mod ui_events;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CursorPosition {
    OutOfScreen,
    InScreen{
        mouse_x: f32,
        mouse_y: f32,
    },
}


pub struct UiEventHandlerContext {}

impl UiEventHandlerContext {
    fn on_window_input(&mut self, event: &winit::event::WindowEvent, components: &mut ComponentTable) {

        let ui_event = match UiEvent::try_from(event) {
            Ok(ev) => ev,
            Err(_) => return,
        };

        let mut callbacks = Vec::new();
        
        for (_entity, transform, listener) in components.query2d_mut::<Transform, UiEventListener>() {
            if let Some(callback) = listener.listener() {
                match callback.on_event(ui_event, transform) {
                    Some(callback) => callbacks.push(callback),
                    None => {},
                }
            }
        }

        for callback in callbacks.into_iter() {
            callback(components);
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
        UiEventHandlerContext {}
    }
}