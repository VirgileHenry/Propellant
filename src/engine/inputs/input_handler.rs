use std::collections::HashMap;

use crate::engine::{
    engine_events::PropellantEvent,
    errors::PResult
};

use super::input_context::InputContext;

pub(crate) mod input_handler_builder;

/// Main event handler of the propellant engine.
/// Must be built from a winit event loop, to allow custom events sending.
pub struct InputHandler {
    event_loop_proxy: winit::event_loop::EventLoopProxy<PropellantEvent>,
    contexts: HashMap<u64, Box<dyn InputContext>>
}


impl InputHandler {
    pub fn send_engine_event(&self, event: PropellantEvent) -> PResult<()> {
        self.event_loop_proxy.send_event(event)?;
        Ok(())
    }

    pub fn request_add_context(&mut self, id: u64) -> PResult<()> {
        self.event_loop_proxy.send_event(PropellantEvent::AddEventContext(id))?;
        Ok(())
    }

    pub fn request_remove_context(&mut self, id: u64) -> PResult<()> {
        self.event_loop_proxy.send_event(PropellantEvent::RemoveEventContext(id))?;
        Ok(())
    }

    pub fn get_context(&mut self, id: u64) -> Option<Box<dyn InputContext>> {
        self.contexts.remove(&id)
    }

    pub fn add_context(&mut self, id: u64, context: Box<dyn InputContext>) {
        self.contexts.insert(id, context);
    }
}
