use crate::{PropellantEngine, PropellantWindow, InputHandler, id};

use super::{inputs::input_system::InputSystem, consts::PROPELLANT_DEBUG_FEATURES};



/// Events to send to the event loop.
#[derive(Debug, Clone)]
pub enum PropellantEvent {
    CloseApplicationRequest,
    SwapchainRecreationRequest,
    AddEventContext(u64),
    RemoveEventContext(u64),
}

impl PropellantEngine {
    pub fn handle_propellant_event(&mut self, event: PropellantEvent, control_flow: &mut winit::event_loop::ControlFlow) {
        match event {
            // engine requested stop
            PropellantEvent::CloseApplicationRequest => control_flow.set_exit(),
            PropellantEvent::SwapchainRecreationRequest => {
                // get to the window, and ask swap chain recreation.
                match self.world.get_singleton_mut::<PropellantWindow>() {
                    Some(window) => {
                        match window.handle_window_resize() {
                            Ok(_) => {},
                            Err(e) => println!("{e}"),
                        };
                    },
                    None => {},
                }
            },
            PropellantEvent::AddEventContext(ctx_id) => match self.world.get_system_and_world_mut(id("input_system")) {
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
            },
            PropellantEvent::RemoveEventContext(ctx_id) => match self.world.get_system_and_world_mut(id("input_system")) {
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
            },
        }
    }
}