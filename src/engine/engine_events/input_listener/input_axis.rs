use crate::InputListener;



pub struct InputAxis {
    keys: [winit::event::VirtualKeyCode; 2],
    state: (bool, bool),
    value: f64,
}

impl InputListener for InputAxis {
    fn send_input(&mut self, input: winit::event::DeviceEvent) {
        match input {
            winit::event::DeviceEvent::Key(winit::event::KeyboardInput {
                virtual_keycode,
                state,
                ..
            }) => match virtual_keycode {
                Some(keycode) => {
                    if self.keys[0] == keycode {
                        self.state.0 = match state {
                            winit::event::ElementState::Pressed => true,
                            winit::event::ElementState::Released => false,
                        };
                    }
                    else if self.keys[1] == keycode {
                        self.state.1 = match state {
                            winit::event::ElementState::Pressed => true,
                            winit::event::ElementState::Released => false,
                        };
                    }
                },
                None => {},
            },
            _ => {},
        }
    }

    fn update(&mut self, _delta: f32) {
        // todo : elasticity and stuff
        match self.state {
            (false, false) | (true, true) => self.value = 0.,
            (false, true) => self.value = 1.,
            (true, false) => self.value = -1.,
        }
    }
}