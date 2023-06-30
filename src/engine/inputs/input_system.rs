use std::mem::swap;

use foundry::{
    AsAny,
    Updatable, System, UpdateFrequency
};

use super::input_context::InputContext;

/// holds the current input context,
/// and performs transmission from raw to context, then from context to game.
#[derive(AsAny)]
pub struct InputSystem {
    current_context: Box<dyn InputContext>,
    current_context_id: u64,
}

impl InputSystem {
    pub fn new(context_id: u64, input_context: Box<dyn InputContext>) -> System {
        System::new(
            Box::new(InputSystem {
                current_context: input_context,
                current_context_id: context_id,
            }),
            UpdateFrequency::PerFrame,
        )
    }

    pub fn handle_device_event(&mut self, device_id: winit::event::DeviceId, input: winit::event::DeviceEvent) {
        self.current_context.handle_raw_input(device_id, input);
    }

    pub fn switch_context(&mut self, id: &mut u64, context: &mut Box<dyn InputContext>) {
        swap(&mut self.current_context, context);
        swap(&mut self.current_context_id, id);
    }

    pub fn on_become_active(&mut self, components: &mut foundry::ComponentTable) {
        self.current_context.on_become_active(components);
    }
}

impl Updatable for InputSystem {
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        self.current_context.update(components, delta);
    }
}