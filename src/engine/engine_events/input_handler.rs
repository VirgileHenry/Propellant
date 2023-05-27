use std::{collections::BTreeMap, any::Any};

use foundry::{Updatable, AsAny};

use crate::engine::errors::PropellantError;

use super::{PropellantEvent, input_listener::InputListener};


/// Main event handler of the propellant engine.
/// Must be built from a winit event loop, to allow custom events sending.
pub struct InputHandler {
    event_loop_proxy: winit::event_loop::EventLoopProxy<PropellantEvent>,
    listeners: BTreeMap<u64, Box<dyn InputListener>>,
}


impl InputHandler {
    /// Creates an event handler form an event loop.
    pub fn empty(event_loop: &winit::event_loop::EventLoop<PropellantEvent>) -> InputHandler {
        InputHandler {
            event_loop_proxy: event_loop.create_proxy(),
            listeners: BTreeMap::new(),
        }
    }

    pub fn from_inputs(event_loop: &winit::event_loop::EventLoop<PropellantEvent>, inputs: BTreeMap<u64, Box<dyn InputListener>>) -> InputHandler {
        InputHandler {
            event_loop_proxy: event_loop.create_proxy(),
            listeners: inputs,
        }
    }

    /// Sends a custom propellant event to the event loop, that will be intercepted in the main loop.
    pub fn send_engine_event(&self, event: PropellantEvent) -> Result<(), PropellantError> {
        self.event_loop_proxy.send_event(event)?;
        Ok(())
    }

    pub fn handle_input(&mut self, input: winit::event::DeviceEvent, _device_id: winit::event::DeviceId) {
        for (_, listener) in self.listeners.iter_mut() {
            listener.send_input(input.clone());
        }
    }


    pub fn update(&mut self, delta: f32) {
        for (_, listener) in self.listeners.iter_mut() {
            listener.update(delta);
        }
    }

    pub fn get_listener<T: 'static>(&self, id: u64) -> Option<&T> {
        (self.listeners.get(&id)? as &dyn Any).downcast_ref()
    } 

}

#[derive(AsAny)]
pub struct InputUpdater {}

impl Updatable for InputUpdater {
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        match components.get_singleton_mut::<InputHandler>() {
            Some(handler) => handler.update(delta),
            None => {},
        }
    }
}

/// Create a BtreeMap with the given sets of inputs.
#[macro_export]
macro_rules! create_inputs {
    ($($id:expr, $input:expr)*) => {
        {
            use std::collections::BTreeMap;
            let mut result: BTreeMap<u64, Box<dyn InputListener>> = BTreeMap::new();
            $(
                result.insert($id, Box::new($input));
            )*
            result
        }
    };
}