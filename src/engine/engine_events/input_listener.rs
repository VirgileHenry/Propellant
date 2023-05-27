pub(crate) mod input_axis;
pub(crate) mod input_button;


/// this is a trait that any struct that want to be used as an input listener should implement.
/// It receives inputs from the event loop, and can be queried for it's state.
pub trait InputListener {
    /// Set the value of the input for a given key.
    /// This will only be called if the listener is interested in the key. 
    fn send_input(&mut self, input: winit::event::DeviceEvent);
    /// update of the game engine.
    fn update(&mut self, delta: f32);
}
