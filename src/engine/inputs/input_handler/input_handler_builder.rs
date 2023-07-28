use std::collections::HashMap;

use crate::{InputContext, engine::engine_events::PropellantEvent, id};

use super::InputHandler;

pub struct InputHandlerBuilder {
    contexts: HashMap<u64, Box<dyn InputContext>>,
    start_context: Vec<u64>,
}

impl InputHandlerBuilder {
    /// Creates an event handler form an event loop.
    pub fn empty() -> InputHandlerBuilder {
        InputHandlerBuilder {
            contexts: HashMap::new(),
            start_context: Vec::new(),
        }
    }

    /// register a new input context.
    /// Note that this context will not be activated by default.
    pub fn with_input_context(
        self,
        id: u64,
        context: Box<dyn InputContext>
    ) -> InputHandlerBuilder {
        let mut contexts = self.contexts;
        contexts.insert(id, context);
        InputHandlerBuilder {
            contexts,
            start_context: self.start_context,
        }
    }

    /// register a new input context that will be activated by default.
    pub fn with_starting_input_context(
        self,
        id: u64,
        context: Box<dyn InputContext>
    ) -> InputHandlerBuilder {
        let mut contexts = self.contexts;
        contexts.insert(id, context);
        let mut start_context = self.start_context;
        start_context.push(id);
        InputHandlerBuilder {
            contexts,
            start_context,
        }
    }

    pub fn build(self, event_loop_proxy: winit::event_loop::EventLoopProxy<PropellantEvent>) -> InputHandler {
        InputHandler {
            event_loop_proxy,
            contexts: self.contexts
        }
    }

    pub fn start_contexts(&self) -> &Vec<u64> {
        &self.start_context
    }
}


#[cfg(feature = "ui")]
impl InputHandlerBuilder {
    /// Creates an event handler form an event loop.
    pub fn with_ui_context(
        self,
    ) -> InputHandlerBuilder {
        let mut contexts = self.contexts;
        contexts.insert(id("ui_context"), Box::new(crate::engine::inputs::common_context::ui_event_context::UiEventHandlerContext::default()));
        InputHandlerBuilder {
            contexts,
            start_context: self.start_context,
        }
    }

    /// register a new input context that will be activated by default.
    pub fn with_starting_ui_context(
        self,
    ) -> InputHandlerBuilder {
        let mut contexts = self.contexts;
        contexts.insert(id("ui_context"), Box::new(crate::engine::inputs::common_context::ui_event_context::UiEventHandlerContext::default()));
        let mut start_context = self.start_context;
        start_context.push(id("ui_context"));
        InputHandlerBuilder {
            contexts,
            start_context,
        }
    }
}