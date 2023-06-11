use foundry::{create_entity, Updatable, System, AsAny, component_iterator};
use propellant::*;





fn main() {
    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_input_handler(create_inputs!(id("jump"), InputButton::new(winit::event::VirtualKeyCode::Space)));
    

    let _cam = create_entity!(engine.world_mut(); Transform::origin().translated(glam::vec3(1., 1., -3.)), Camera::main(800., 450., 0.1, 100., 2.));
    let _cube = create_entity!(engine.world_mut(); Transform::origin(), MeshRendererBuilder::new(Mesh::cube(1.), Material::default()));

    engine.world_mut().register_system(CamMover::new(), 11);


    engine.main_loop();
}

#[derive(AsAny)]
struct CamMover {
    timer: f32,
}

impl CamMover {
    pub fn new() -> System {
        System::new(Box::new(CamMover{ timer: 0. }), foundry::UpdateFrequency::PerFrame)
    }
}

impl Updatable for CamMover {
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        self.timer += delta;
        for (tf, _cam) in component_iterator!(components; mut Transform, Camera) {
            tf.set_position(glam::Vec3::new(1., self.timer.sin(), -3.));
        }
    }
}