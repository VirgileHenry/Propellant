use foundry::create_entity;
use propellant::*;





fn main() {
    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_input_handler(create_inputs!(id("jump"), InputButton::new(winit::event::VirtualKeyCode::Space)));
    

    let _cube = create_entity!(engine.world_mut(); Transform::origin(), MeshRendererBuilder::new(Mesh::cube(0.3), Material::default()));

    engine.main_loop();
}
