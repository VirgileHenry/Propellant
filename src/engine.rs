
use foundry::World;
use crate::{id, InputHandlerBuilder};

use self::{
    engine_events::PropellantEvent,
    window::{
        PropellantWindow,
        window_builder::PropellantWindowBuilder
    },
    errors::{PResult, PropellantError},
    flags::{
        RequireResourcesLoadingFlag,
        RequireSceneRebuildFlag
    },
    resources::ProppellantResources,
    inputs::input_system::InputSystem, 
    consts::PROPELLANT_DEBUG_FEATURES, renderer::graphic_pipeline::uniform::frame_uniform::ui_resolution::UiResolution,
};

pub(crate) mod common_components;
pub(crate) mod common_systems;
pub(crate) mod consts;
pub(crate) mod engine_events;
pub(crate) mod errors;
pub(crate) mod flags;
pub(crate) mod inputs;
pub(crate) mod lights;
pub(crate) mod material;
pub(crate) mod mesh;
pub(crate) mod renderer;
pub(crate) mod resources;
pub(crate) mod transform;
pub(crate) mod window;

#[cfg(feature = "ui")]
pub(crate) mod ui;


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
    pub fn with_window(mut self) -> PResult<PropellantEngine> {
        let event_loop = self.event_loop.take().unwrap();
        // register the rendering system.
        let window = PropellantWindowBuilder::default().build(&event_loop)?;
        self.world.register_system(window.into(), id("window"));
        self.event_loop = Some(event_loop);
        // marks the scene need building
        self.world.add_singleton(RequireSceneRebuildFlag);
        Ok(self)
    }

    /// Adds a window builded with the given builder to the engine, and open the window.
    pub fn with_builded_window(mut self, builder: PropellantWindowBuilder) -> PResult<PropellantEngine> {
        let event_loop = self.event_loop.take().unwrap();
        let window = builder.build(&event_loop)?;
        // register the rendering system.
        self.world.register_system(window.into(), id("window"));
        self.event_loop = Some(event_loop);
        // marks the scene need building
        self.world.add_singleton(RequireSceneRebuildFlag);
        Ok(self)
    }

    /// Adds a mesh library to the engine, and add the mesh lib need rebuild flag.
    pub fn with_resources(mut self, resources: ProppellantResources) -> PropellantEngine {
        self.world.add_singleton(resources);
        self.world.add_singleton(RequireResourcesLoadingFlag::ALL);
        self
    }

    /// Add an input handler to the engine, and register the input system.
    /// The start context id must a id of a input context in the input handler.
    pub fn with_input_handler(mut self, input_handler: InputHandlerBuilder) -> PResult<PropellantEngine> {
        let input_system = InputSystem::new();
        // prepare register of start event contexts
        let event_proxy = match &self.event_loop {
            Some(proxy) => proxy.create_proxy(),
            None => return Err(PropellantError::Custom("No event loop present.".to_string())),
        };
        for ctx_id in input_handler.start_contexts().iter() {
            event_proxy.send_event(PropellantEvent::AddEventContext(*ctx_id))?;
        }
        // register both the input system and the input handler.
        self.world.register_system(input_system, id("input_system"));
        self.world.add_singleton(input_handler.build(match &self.event_loop {
            Some(proxy) => proxy.create_proxy(),
            None => return Err(PropellantError::NoResources),
        }));
        Ok(self)
    }

    #[cfg(feature = "ui")]
    /// Add a ui resolution to the engine, and register it. This can only be called if a window is present.
    pub fn with_ui_resolution(mut self, resolution: f32) -> PResult<PropellantEngine> {
        let (screen_width, screen_height) = match self.world.get_system(id("window")) {
            Some(window) => match window.try_get_updatable::<PropellantWindow>() {
                Some(prop_window) => prop_window.window_inner_size(),
                None => return Err(PropellantError::Custom(String::from("Unable to add a ui resolution without a window."))),
            },
            None => return Err(PropellantError::Custom(String::from("Unable to add a ui resolution without a window."))),
        };
        self.world.add_singleton(UiResolution {
            resolution,
            screen_width,
            screen_height,
        });
        Ok(self)
    }

    /// Main loop of the app.
    pub fn main_loop(mut self) {

        let event_loop = match self.event_loop.take() {
            Some(el) => el,
            None => return, // no event loop, can't loop and return
        };

        event_loop.run(move |event, _, mut control_flow| {
            // better for games and continuous apps.
            control_flow.set_poll();

            match event {
                // redirect windows events to the window
                winit::event::Event::WindowEvent { event, .. } => {
                    match self.world.get_system_and_world_mut(id("input_system")) {
                        Some((input_system_wrapper, components)) => match input_system_wrapper.try_get_updatable_mut::<InputSystem>() {
                            Some(input_system) => input_system.handle_window_event(&event, components),
                            None => {
                                if PROPELLANT_DEBUG_FEATURES {
                                    println!("[PROPELLANT DEBUG] Unable to downcast system registered as 'input handler' to InputSystem.");
                                }
                            }
                        },
                        None => {},
                    };
                    match self.world.get_system_and_world_mut(id("window")) {
                        Some((window_system, comps)) => match window_system.try_get_updatable_mut::<PropellantWindow>() {
                            Some(window) => window.handle_event(event, control_flow, comps),
                            None => {
                                if PROPELLANT_DEBUG_FEATURES {
                                    println!("[PROPELLANT DEBUG] Unable to downcast system registered as 'window' to PropellantWindow.");
                                }
                            },
                        },
                        None => {},
                    };
                },
                // device events are treated by the input handler, if any
                winit::event::Event::DeviceEvent { device_id, event } => {
                    match self.world.get_system_and_world_mut(id("input_system")) {
                        Some((input_system_wrapper, components)) => match input_system_wrapper.try_get_updatable_mut::<InputSystem>() {
                            Some(input_system) => input_system.handle_device_event(device_id, event, components),
                            None => {
                                if PROPELLANT_DEBUG_FEATURES {
                                    println!("[PROPELLANT DEBUG] Unable to downcast system registered as 'input handler' to InputSystem.");
                                }
                            }
                        },
                        None => {},
                    };
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
                winit::event::Event::UserEvent(event) => self.handle_propellant_event(event, &mut control_flow),
                winit::event::Event::LoopDestroyed => {
                    
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


impl Drop for PropellantEngine {
    fn drop(&mut self) {
        // clean up the engine's resources
        self.clean_up();
    }
}