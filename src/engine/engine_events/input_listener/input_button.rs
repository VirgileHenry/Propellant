use crate::InputListener;


/// Listen to a single key, and set  it's state to true when the key is pressed.
pub struct InputButton {
    key: winit::event::VirtualKeyCode,
    state: bool,
}

impl InputButton {
    pub fn new(key: winit::event::VirtualKeyCode) -> InputButton {
        InputButton { 
            key,
            state: false
        }
    }
}

impl InputListener for InputButton {
    fn send_input(&mut self, input: winit::event::DeviceEvent) {
        match input {
            winit::event::DeviceEvent::Key(winit::event::KeyboardInput {
                virtual_keycode,
                state,
                ..
            }) => {
                if match virtual_keycode {
                    Some(keycode) => self.key == keycode,
                    None => false,
                } {
                    // using destructuring, match only if the button is the one we registered
                    self.state = match state {
                        winit::event::ElementState::Pressed => true,
                        winit::event::ElementState::Released => false,
                    }
                }
            },
            _ => {},
        }
    }

    fn update(&mut self, _delta: f32) {}
}