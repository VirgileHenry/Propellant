use foundry::ComponentTable;

use crate::{Transform, engine::inputs::common_context::ui_event_context::ui_events::UiEvent};

pub struct UiEventListener {
    callback: Option<Box<dyn UiListenerCallback>>,
}

pub trait UiListenerCallback {
    /// Called by the ui context whenever a ui event is triggered.
    /// This functions return a callback that we want the system to call after the event has been handled.
    fn on_event(&mut self, event: UiEvent, transform: &mut Transform) -> Option<Box<dyn Fn(&mut ComponentTable)>>;
    fn update(&mut self, transform: &mut Transform, delta: f32);
}

impl UiEventListener {
    pub fn new<T: UiListenerCallback + 'static>(listener: T) -> UiEventListener {
        UiEventListener {
            callback: Some(Box::new(listener)),
        }
    }

    pub fn listener(&mut self) -> &mut Option<Box<dyn UiListenerCallback>> {
        &mut self.callback
    }

}

