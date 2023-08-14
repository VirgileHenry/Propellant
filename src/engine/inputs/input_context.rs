use foundry::ComponentTable;

use crate::{
    PropellantEngine,
    engine::consts::PROPELLANT_DEBUG_FEATURES,
    InputHandler
};



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
        match self.world.get_singleton_mut::<InputHandler>() {
            Some(handler) => match handler.get_context(ctx_id) {
                Some(ctx) => self.input_system.register_context(ctx_id, ctx),
                None => if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] [INPUTS] Tried to register context of id {ctx_id} but such context does not exist.");
                },
            },
            None => if PROPELLANT_DEBUG_FEATURES {
                println!("[PROPELLANT DEBUG] [INPUTS] Tried to register context but no input handler singleton");
            },
        }
    }
    pub fn remove_input_context(&mut self, ctx_id: u64) {
        let context = match self.input_system.remove_context(ctx_id) {
            Some(ctx) => ctx,
            None => {
                if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] [INPUTS] Tried to remove context of id {ctx_id} but such context is not currently active. Nothing will change.");
                }
                return;
            }
        };
        match self.world.get_singleton_mut::<InputHandler>() {
            Some(handler) => handler.add_context(ctx_id, context),
            None => if PROPELLANT_DEBUG_FEATURES {
                println!("[PROPELLANT DEBUG] [INPUTS] Removing input context, but no input handler found in world. Input context will be lost.")
            }
        }
    }
}