use foundry::ComponentTable;

use crate::{PropellantEngine, engine::consts::PROPELLANT_DEBUG_FEATURES, InputHandler, id};

use super::input_system::InputSystem;



/// An input context is a set of input states, that can be bound to from the raw inputs.
/// we can register multiple contexts and swich between them.
pub trait InputContext {
    /// How this context should handle raw inputs. Can be called multiple times per frame.
    fn handle_device_input(&mut self, device_id: winit::event::DeviceId, input: winit::event::DeviceEvent, components: &mut ComponentTable);
    /// How this context should handle window inputs. Can be called multiple times per frame.
    fn handle_window_input(&mut self, input: &winit::event::WindowEvent, components: &mut ComponentTable);
    /// How this input handler should send inputs to the game. Called once per frame.
    fn update(&mut self, components: &mut ComponentTable, delta: f32);
    /// Called when this context becomes the active one.
    fn on_become_active(&mut self, components: &mut ComponentTable);
}


impl PropellantEngine {
    pub fn add_input_context(&mut self, ctx_id: u64) {
        match self.world.get_system_and_world_mut(id("input_system")) {
            Some((input_system_wrapper, comps)) 
                => match (input_system_wrapper.try_get_updatable_mut::<InputSystem>(), comps.get_singleton_mut::<InputHandler>()) {
                (Some(input_system), Some(input_handler)) => {
                    match input_handler.get_context(ctx_id) {
                        Some(context) => input_system.register_context(ctx_id, context),
                        None => if PROPELLANT_DEBUG_FEATURES {
                            println!("[PROPELLANT DEBUG] Unable to find context with id {} in input handler.", ctx_id);
                        }
                    }
                },
                _ => if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] Unable to downcast system registered as 'input handler' to InputSystem.");
                }                                
            },
            None => {},
        }
    }
    pub fn remove_input_context(&mut self, ctx_id: u64) {
        match self.world.get_system_and_world_mut(id("input_system")) {
            Some((input_system_wrapper, comps)) 
                => match (input_system_wrapper.try_get_updatable_mut::<InputSystem>(), comps.get_singleton_mut::<InputHandler>()) {
                (Some(input_system), Some(input_handler)) => {
                    match input_system.remove_context(ctx_id) {
                        Some(context) => input_handler.add_context(ctx_id, context),
                        None => if PROPELLANT_DEBUG_FEATURES {
                            println!("[PROPELLANT DEBUG] Unable to find context with id {} in input handler.", ctx_id);
                        }
                    }
                },
                _ => if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] Unable to downcast system registered as 'input handler' to InputSystem.");
                }                                
            },
            None => {},
        }
    }
}