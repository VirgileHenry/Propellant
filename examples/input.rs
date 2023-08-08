use foundry::{create_entity, ComponentTable};
use propellant::*;


// create a custom input context
struct InputContext1 {
    space_pressed: bool,
    ask_switch: bool,
}
struct InputContext2 {
    space_pressed: bool,
    ask_switch: bool,
}


impl InputContext for InputContext1 {
    fn handle_device_input(&mut self, _device_id: winit::event::DeviceId, input: winit::event::DeviceEvent, _components: &mut ComponentTable) {
        match input {
            winit::event::DeviceEvent::Key(winit::event::KeyboardInput {
                state, virtual_keycode, ..
            }) => match (state, virtual_keycode) {
                (winit::event::ElementState::Pressed, Some(winit::event::VirtualKeyCode::Space)) => {
                    self.space_pressed = true;
                }
                (winit::event::ElementState::Released, Some(winit::event::VirtualKeyCode::Space)) => {
                    self.space_pressed = false;
                }
                (winit::event::ElementState::Pressed, Some(winit::event::VirtualKeyCode::Return)) => {
                    self.ask_switch = true;
                }
                _ => {}
            },
            _ => {},
        }
    }
    fn handle_window_input(&mut self, _input: &winit::event::WindowEvent, _components: &mut ComponentTable) {
        // ignore 
    }
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        // if space is being pressed, rotate the cubes.
        if self.space_pressed {
            for (_entitiy, transform, _) in components.query2d_mut::<Transform, InstancedMeshRenderer<PhongMaterial>>() {
                transform.rotate(glam::Quat::from_rotation_y(delta));
            }
        }
        if self.ask_switch {
            self.ask_switch = false;
            println!("Switching to context 2");
            match components.get_singleton::<InputHandler>() {
                Some(handler) => {
                    handler.send_engine_event(PropellantEvent::AddEventContext(id("ih2"))).unwrap();
                    handler.send_engine_event(PropellantEvent::RemoveEventContext(id("ih1"))).unwrap();
                },
                None => {}
            }
        }
    }
    fn on_become_active(&mut self, _components: &mut foundry::ComponentTable) {
        // reset our values
        self.space_pressed = false;
        self.ask_switch = false;
    }
}




impl InputContext for InputContext2 {
    fn handle_device_input(&mut self, _device_id: winit::event::DeviceId, input: winit::event::DeviceEvent, _components: &mut ComponentTable) {
        match input {
            winit::event::DeviceEvent::Key(winit::event::KeyboardInput {
                state, virtual_keycode, ..
            }) => match (state, virtual_keycode) {
                (winit::event::ElementState::Pressed, Some(winit::event::VirtualKeyCode::Space)) => {
                    self.space_pressed = true;
                }
                (winit::event::ElementState::Released, Some(winit::event::VirtualKeyCode::Space)) => {
                    self.space_pressed = false;
                }
                (winit::event::ElementState::Pressed, Some(winit::event::VirtualKeyCode::Return)) => {
                    self.ask_switch = true;
                }
                _ => {}
            },
            _ => {},
        }
    }
    fn handle_window_input(&mut self, _input: &winit::event::WindowEvent, _components: &mut ComponentTable) {
        // ignore 
    }
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        // if space is being pressed, rotate the cubes.
        if self.space_pressed {
            for (_entitiy, transform, _) in components.query2d_mut::<Transform, InstancedMeshRenderer<PhongMaterial>>() {
                transform.rotate(glam::Quat::from_rotation_y(-delta));
            }
        }
        if self.ask_switch {
            self.ask_switch = false;
            println!("Switching to context 1");
            match components.get_singleton::<InputHandler>() {
                Some(handler) => {
                    handler.send_engine_event(PropellantEvent::RemoveEventContext(id("ih2"))).unwrap();
                    handler.send_engine_event(PropellantEvent::AddEventContext(id("ih1"))).unwrap();
                },
                None => {}
            }
        }
    }
    fn on_become_active(&mut self, _components: &mut foundry::ComponentTable) {
        // reset our values
        self.space_pressed = false;
        self.ask_switch = false;
    }
}




fn main() {

    let input_handler = InputHandlerBuilder::empty()
        .with_starting_input_context(id("ih1"), Box::new(InputContext1 {
            space_pressed: false,
            ask_switch: false,
        }))
        .with_input_context(id("ih2"), Box::new(InputContext2 {
            space_pressed: false,
            ask_switch: false,
        }));

    let mut resources = ProppellantResources::default();
    resources.meshes_mut().register_mesh(id("cube"), Mesh::cube(1.0));

    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_resources(resources)
        .with_input_handler(input_handler).unwrap();

    let _cam = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., -3., -4.)),
        Camera::main_perspective(800., 450., 0.1, 100., 1.5)
    );

    // sun
    engine.world_mut().add_singleton(DirectionnalLight::new(
        glam::vec3(1., 1., 1.),
        glam::vec3(1., 1., 1.),
        glam::vec3(-1., -1., -1.)
    ));

    let _cube = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(-1., 1., 0.)),
        InstancedMeshRenderer::new(
            id("cube"),
            PhongMaterial::default().colored(glam::vec3(0.6, 0., 0.))
        )
    );
    let _cube = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(2., 0., 0.)),
        InstancedMeshRenderer::new(
            id("cube"),
            PhongMaterial::default().colored(glam::vec3(0., 0.6, 0.))
        )
    );

    engine.main_loop();

}

