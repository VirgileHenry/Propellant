use foundry::World;
use winit::event_loop;

use crate::{
    PropellantWindow,
    PropellantWindowBuilder,
    utils::builder::HasBuilder,
    PropellantEngine,
    PropellantEvent,
    PropellantResources,
    PropellantFlag,
    resource_loading::RequireResourcesLoadingFlag,
};
#[cfg(feature = "inputs")]
use crate::{
    InputHandlerBuilder,
    InputHandler,
};

use super::{errors::PResult, engine_events::PropellantEventSender};



pub struct PropellantEngineBuilder {
    world: World,
    #[cfg(feature = "window")]
    window: PropellantWindowBuilder,
    #[cfg(feature = "resources")]
    resources: PropellantResources,
    #[cfg(feature = "inputs")]
    input_handler: InputHandlerBuilder,
}

impl HasBuilder for PropellantEngine {
    type Builder = PropellantEngineBuilder;
    fn builder() -> Self::Builder {
        PropellantEngineBuilder {
            world: World::default(),
            #[cfg(feature = "window")]
            window: PropellantWindow::builder(),
            #[cfg(feature = "resources")]
            resources: PropellantResources::default(),
            #[cfg(feature = "inputs")]
            input_handler: InputHandler::builder(),
        }
    }
}

#[cfg(feature = "window")]
impl PropellantEngineBuilder {
    pub fn with_window(self, window: PropellantWindowBuilder) -> PropellantEngineBuilder {
        PropellantEngineBuilder {
            world: self.world,
            window,
            #[cfg(feature = "resources")]
            resources: self.resources,
            #[cfg(feature = "inputs")]
            input_handler: self.input_handler,
        }
    }
}

#[cfg(feature = "resources")]
impl PropellantEngineBuilder {
    pub fn with_resources(self, resources: PropellantResources) -> PropellantEngineBuilder {
        PropellantEngineBuilder {
            world: self.world,
            resources,
            #[cfg(feature = "window")]
            window: self.window,
            #[cfg(feature = "inputs")]
            input_handler: self.input_handler,
        }
    }
}

impl PropellantEngineBuilder {
    #[cfg(feature = "inputs")]
    pub fn with_input_handler(self, input_handler: InputHandlerBuilder) -> PropellantEngineBuilder {
        PropellantEngineBuilder { 
            world: self.world,
            #[cfg(feature = "inputs")]
            window: self.window,
            #[cfg(feature = "resources")]
            resources: self.resources,
            input_handler,
        }
    }
}

impl PropellantEngineBuilder {
    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Build the builder into a engine and an event loop.
    fn build(self) -> PResult<(PropellantEngine, winit::event_loop::EventLoop<PropellantEvent>)> {
        let event_loop = event_loop::EventLoopBuilder::with_user_event().build();
        #[cfg(feature = "window")]
        let window = self.window.build(&event_loop)?;

        let mut world = self.world;
        #[cfg(feature = "resources")]
        world.add_singleton(self.resources);

        #[cfg(feature = "inputs")]
        let (
            input_handler,
            input_system
        ) = self.input_handler.build(event_loop.create_proxy());
        #[cfg(feature = "inputs")]
        world.add_singleton(input_handler);

        Ok((PropellantEngine {
            world,
            last_frame_update: std::time::Instant::now(),
            #[cfg(feature = "window")]
            window,
            #[cfg(feature = "inputs")]
            input_system,
        }, event_loop))
    }
    
    /// Build the engine and start the main loop.
    pub fn main_loop(self) -> PResult<()> {

        let (mut engine, event_loop) = self.build()?;

        // create the sender and sent startup events
        let event_sender = PropellantEventSender::new(event_loop.create_proxy());
        event_sender.send(PropellantEvent::HandleEngineFlag(PropellantFlag::RequireSceneRebuild))?;
        event_sender.send(PropellantEvent::HandleEngineFlag(PropellantFlag::RequireResourcesLoading(RequireResourcesLoadingFlag::ALL)))?;
        engine.world_mut().add_singleton(event_sender);

        event_loop.run(move |event, _, control_flow| {
            engine.main_loop(event, control_flow);
        });
    }
}