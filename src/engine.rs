use std::collections::BTreeMap;

use foundry::World;
use crate::id;

use self::{
    engine_events::{
        PropellantEvent,
        input_handler::InputHandler,
        input_listener::InputListener
    },
    window::{
        PropellantWindow,
        window_builder::PropellantWindowBuilder
    },
    errors::PropellantError,
};

pub(crate) mod consts;
pub(crate) mod engine_events;
pub(crate) mod errors;
pub(crate) mod material;
pub(crate) mod mesh;
pub(crate) mod renderer;
pub(crate) mod transform;
pub(crate) mod window;



/// An instance of the propellant game engine.
/// This is a wrapper around a foundry ECS world, with easy implementations of add-ons.
pub struct PropellantEngine {
    /// Inner foundry world, containing all the entities and systems.
    world: World,
    /// Instant of the last called update.
    last_frame_update: std::time::Instant,
    /// the event loop used to run the main app.
    /// even with no windows, we base our logic on it as we can have custom events.
    /// The option allows to take() it for running the main loop, and put it back.
    event_loop: Option<winit::event_loop::EventLoop<PropellantEvent>>,
}

impl Default for PropellantEngine {
    fn default() -> Self {
        // create the event loop.
        let mut event_loop_builder = winit::event_loop::EventLoopBuilder::<PropellantEvent>::with_user_event();

        PropellantEngine {
            event_loop: Some(event_loop_builder.build()),
            last_frame_update: std::time::Instant::now(),
            world: World::default()
        }
    }
}

// impl of our engine
impl PropellantEngine {
    /// Create a new empty instance of the propellant engine
    pub fn new() -> PropellantEngine {
        PropellantEngine::default()
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    /// Adds a default window to the engine, and open the window.
    pub fn with_window(mut self) -> Result<PropellantEngine, PropellantError> {
        let event_loop = self.event_loop.take().unwrap();
        let window = PropellantWindowBuilder::default().build(&event_loop)?;
        self.world.register_system(window.into(), id("window"));
        self.event_loop = Some(event_loop);
        // register the rendering system.
        Ok(self)
    }

    /// Adds a window builded with the given builder to the engine, and open the window.
    pub fn with_builded_window(mut self, builder: PropellantWindowBuilder) -> Result<PropellantEngine, PropellantError> {
        let event_loop = self.event_loop.take().unwrap();
        let window = builder.build(&event_loop)?;
        self.world.register_system(window.into(), id("window"));
        self.event_loop = Some(event_loop);
        // register the rendering system.
        Ok(self)
    }

    /// Adds a event handler singletin to the engine
    pub fn with_empty_input_handler(mut self) -> PropellantEngine {
        let event_loop = self.event_loop.take().unwrap();
        self.world.add_singleton(InputHandler::empty(&event_loop));
        self.event_loop = Some(event_loop);
        self
    }

    /// Adds an event handler with the specified input listeners to the engine.
    pub fn with_input_handler(mut self, inputs: BTreeMap<u64, Box<dyn InputListener>>) -> PropellantEngine {
        let event_loop = self.event_loop.take().unwrap();
        self.world.add_singleton(InputHandler::from_inputs(&event_loop, inputs));
        self.event_loop = Some(event_loop);
        self
    }

    /// Main loop of the app.
    pub fn main_loop(mut self) {
        let event_loop = match self.event_loop.take() {
            Some(el) => el,
            None => return, // no event loop, can't loop and return
        };

        event_loop.run(move |event, _, control_flow| {
            // better for games and continuous apps.
            control_flow.set_poll();

            match event {
                // redirect windows events to the window
                winit::event::Event::WindowEvent { event, .. } => {
                    match self.world.get_system_and_world_mut(id("window")) {
                        Some((window_system, comps)) => match window_system.try_get_updatable::<PropellantWindow>() {
                            Some(window) => window.handle_event(event, control_flow, comps),
                            None => {},
                        },
                        None => {},
                    }
                },
                // device events are treated by the input handler, if any
                winit::event::Event::DeviceEvent { device_id, event } => {
                    match self.world.get_singleton_mut::<InputHandler>() {
                        Some(input_handler) => input_handler.handle_input(event, device_id),
                        None => {},
                    }
                }
                // main events cleared is the app code update (all events are pocessed)
                winit::event::Event::MainEventsCleared => {
                    // Application update code.
                    self.engine_update();

                    // request redraw on the window, if there is one
                    match self.world.get_singleton::<PropellantWindow>() {
                        Some(window) => window.request_redraw(),
                        None => {},
                    }
                },
                // handle engine events
                winit::event::Event::UserEvent(prop_event) => match prop_event {
                    // engine requested stop
                    PropellantEvent::CloseApplicationRequest => control_flow.set_exit(),
                    PropellantEvent::SwapchainRecreationRequest => {
                        // get to the window, and ask swap chain recreation.
                        match self.world.get_singleton_mut::<PropellantWindow>() {
                            Some(window) => {
                                match window.vk_swapchain_recreation_request() {
                                    Ok(_) => {},
                                    Err(e) => println!("[PROPELLANT ERROR] Error while recreating swapchain: {:?}", e),
                                };
                            },
                            None => {},
                        }
                    }
                }
                winit::event::Event::LoopDestroyed => {
                    self.clean_up();
                }
                _ => ()
            }
        });
    }

    /// single update of the whole engine. Calls update on the world, managing delta time.
    fn engine_update(&mut self) {
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame_update);
        self.last_frame_update = now;
        self.world.update(delta.as_secs_f32());
    }

    /// Clean up the engine, destroying all the resources.
    fn clean_up(&mut self) {
        // clean up the window: need to destroy allocated gpu ressources.
        match self.world.remove_system(id("window")) {
            Some(mut window) => match window.try_get_updatable_mut::<PropellantWindow>() {
                Some(prop_window) => prop_window.world_clean_up(&mut self.world),
                None => {},
            }
            None => {/* no window on the engine. */}
        }
    }
}


// other impls 
