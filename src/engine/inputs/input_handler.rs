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
    contexts: Vec<(u64, Box<dyn InputContext>)>
}


impl InputHandler {
    pub fn send_engine_event(&self, event: PropellantEvent) -> PResult<()> {
        self.event_loop_proxy.send_event(event)?;
        Ok(())
    }

    pub fn switch_context(&mut self, id: u64) -> PResult<()> {
        self.event_loop_proxy.send_event(PropellantEvent::SwitchInputContext(id))?;
        Ok(())
    }

    pub fn get_context(&mut self, id: u64) -> Option<(&mut u64, &mut Box<dyn InputContext>)> {
        for (context_id, context) in self.contexts.iter_mut() {
            if *context_id == id {
                return Some((context_id, context));
            }
        }
        None
    }
}
