use crate::{InputContext, engine::engine_events::PropellantEvent};

use super::InputHandler;



pub struct InputHandlerBuilder {
    contexts: Vec<(u64, Box<dyn InputContext>)>
}

impl InputHandlerBuilder {
    /// Creates an event handler form an event loop.
    pub fn empty() -> InputHandlerBuilder {
        InputHandlerBuilder {
            contexts: Vec::new(),
        }
    }

    pub fn with_input_context(
        self,
        id: u64,
        context: Box<dyn InputContext>
    ) -> InputHandlerBuilder {
        let mut contexts = self.contexts;
        contexts.push((id, context));
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

    pub fn remove_context(&mut self, id: u64) -> Option<Box<dyn InputContext>> {
        let mut index = None;
        for (i, (context_id, _)) in self.contexts.iter().enumerate() {
            if *context_id == id {
                index = Some(i);
                break;
            }
        }
        if let Some(i) = index {
            Some(self.contexts.remove(i).1)
        } else {
            None
        }
    }
}