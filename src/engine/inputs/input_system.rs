use std::collections::HashMap;

use foundry::{
    AsAny,
    Updatable, System, UpdateFrequency, ComponentTable
};

use super::input_context::InputContext;

/// holds the current input context,
/// and performs transmission from raw to context, then from context to game.
#[derive(AsAny)]
pub struct InputSystem {
    active_contexts: HashMap<u64, Box<dyn InputContext>>,
}

impl InputSystem {
    pub fn new() -> System {
        System::new(
            Box::new(InputSystem {
                active_contexts: HashMap::new(),
            }),
            UpdateFrequency::PerFrame,
        )
    }
    
    pub fn handle_device_event(&mut self, device_id: winit::event::DeviceId, input: winit::event::DeviceEvent, components: &mut ComponentTable) {
        for (_, context) in self.active_contexts.iter_mut() {
            context.handle_device_input(device_id, input.clone(), components);
        }
    }
    
    pub fn handle_window_event(&mut self, input: &winit::event::WindowEvent, components: &mut ComponentTable) {
        for (_, context) in self.active_contexts.iter_mut() {
            context.handle_window_input(input, components);
        }
    }

    pub fn register_context(&mut self, id: u64, context: Box<dyn InputContext>) {
        self.active_contexts.insert(id, context);
    }

    pub fn remove_context(&mut self, id: u64) -> Option<Box<dyn InputContext>> {
        self.active_contexts.remove(&id)
    }
}

impl Updatable for InputSystem {
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        for (_, context) in self.active_contexts.iter_mut() {
            context.update(components, delta);
        }
    }
}