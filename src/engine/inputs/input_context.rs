use foundry::ComponentTable;



/// An input context is a set of input states, that can be bound to from the raw inputs.
/// we can register multiple contexts and swich between them.
pub trait InputContext {
    /// How this context should handle raw inputs. Can be called multiple times per frame.
    fn handle_raw_input(&mut self, device_id: winit::event::DeviceId, input: winit::event::DeviceEvent);
    /// How this input handler should send inputs to the game. Called once per frame.
    fn update(&mut self, components: &mut ComponentTable, delta: f32);
    /// Called when this context becomes the active one.
    fn on_become_active(&mut self, components: &mut ComponentTable);
}
