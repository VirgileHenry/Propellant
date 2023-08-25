
use foundry::World;
use crate::{
    PropellantFlag,
    resource_loading::RequireResourcesLoadingFlag
};

use self::{
    engine_events::{PropellantEvent, PropellantEventSenderExt},
    window::PropellantWindow,
    errors::PResult,
    resources::PropellantResources, 
};

#[cfg(feature = "inputs")]
use self::inputs::input_system::InputSystem;

pub(crate) mod common_components;
pub(crate) mod common_systems;
pub(crate) mod consts;
pub(crate) mod engine_builder;
pub(crate) mod engine_events;
pub(crate) mod errors;
pub(crate) mod flags;
#[cfg(feature = "inputs")]
pub(crate) mod inputs;
pub(crate) mod lights;
pub(crate) mod material;
pub(crate) mod mesh;
pub(crate) mod renderer;
pub(crate) mod resources;
pub(crate) mod transform;
#[cfg(feature = "ui")]
pub(crate) mod ui;
#[cfg(feature = "window")]
pub(crate) mod window;



/// An instance of the propellant game engine.
/// This is a wrapper around a foundry ECS world, with easy implementations of add-ons.
pub struct PropellantEngine {
    /// Inner foundry world, containing all the entities and systems.
    world: World,
    /// Instant of the last called update.
    last_frame_update: std::time::Instant,
    /// If we have the window feature, this is the window handle.
    #[cfg(feature = "window")]
    window: PropellantWindow,
    #[cfg(feature = "inputs")]
    input_system: InputSystem,
}

// impl of our engine
impl PropellantEngine {

    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    #[cfg(feature = "window")]
    pub fn world_and_window(&self) -> (&World, &PropellantWindow) {
        (&self.world, &self.window)
    }

    #[cfg(feature = "window")]
    pub fn world_and_window_mut(&mut self) -> (&mut World, &mut PropellantWindow) {
        (&mut self.world, &mut self.window)
    }

    /// Adds a mesh library to the engine, and add the mesh lib need rebuild flag.
    pub fn with_resources(mut self, resources: PropellantResources) -> PResult<PropellantEngine> {
        self.world.add_singleton(resources);
        self.world.send_flag(PropellantFlag::RequireResourcesLoading(RequireResourcesLoadingFlag::ALL))?;
        Ok(self)
    }

    pub fn main_loop(
        &mut self,
        event: winit::event::Event<PropellantEvent>,
        control_flow: &mut winit::event_loop::ControlFlow
    ) {
        // better for games and continuous apps.
        control_flow.set_poll();
        match event {
            // redirect windows events to the window
            winit::event::Event::WindowEvent { event, .. } => {
                #[cfg(feature = "inputs")]
                self.input_system.handle_window_event(&event, &mut self.world);
                #[cfg(feature = "window")]
                self.window.handle_event(event, control_flow, &mut self.world);
            },
            // device events are treated by the input handler, if any
            winit::event::Event::DeviceEvent { device_id, event } => {
                #[cfg(feature = "inputs")]
                self.input_system.handle_device_event(device_id, event, &mut self.world);
            }
            // main events cleared is the app code update (all events are pocessed)
            winit::event::Event::MainEventsCleared => self.engine_update(),
            #[cfg(feature = "window")]
            winit::event::Event::RedrawRequested(_) => self.window.render(&mut self.world),
            // handle engine events
            winit::event::Event::UserEvent(event) => self.handle_propellant_event(event, control_flow),
            winit::event::Event::LoopDestroyed => {
                
            }
            _ => ()
        }
    }

    /// single update of the whole engine. Calls update on the world, managing delta time.
    fn engine_update(&mut self) {
        let now = std::time::Instant::now();
        let delta = now.duration_since(self.last_frame_update);
        self.last_frame_update = now;
        self.world.update(delta.as_secs_f32());

        #[cfg(feature = "inputs")]
        self.input_system.update_contexts(&mut self.world, delta.as_secs_f32());

        #[cfg(feature = "window")]
        self.window.request_redraw();
    }

    /// Clean up the engine, destroying all the resources.
    fn clean_up(&mut self) {
        // clean up the window: need to destroy allocated gpu ressources.
        #[cfg(feature = "window")]
        self.window.world_clean_up(&mut self.world);
    }
}


impl Drop for PropellantEngine {
    fn drop(&mut self) {
        // clean up the engine's resources
        self.clean_up();
    }
}