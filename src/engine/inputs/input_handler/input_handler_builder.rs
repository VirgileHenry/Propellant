use std::collections::HashMap;

use crate::{InputContext, engine::engine_events::PropellantEvent};

use super::InputHandler;



pub struct InputHandlerBuilder {
    contexts: HashMap<u64, Box<dyn InputContext>>
}

impl InputHandlerBuilder {
    /// Creates an event handler form an event loop.
    pub fn empty() -> InputHandlerBuilder {
        InputHandlerBuilder {
            contexts: HashMap::new(),
        }
    }

    pub fn with_input_context(
        self,
        id: u64,
        context: Box<dyn InputContext>
    ) -> InputHandlerBuilder {
        let mut contexts = self.contexts;
        contexts.insert(id, context);
        InputHandlerBuilder {
            contexts
        }
    }

    pub fn build(self, event_loop_proxy: winit::event_loop::EventLoopProxy<PropellantEvent>) -> InputHandler {
        InputHandler {
            event_loop_proxy,
            contexts: self.contexts
        }
    }
}