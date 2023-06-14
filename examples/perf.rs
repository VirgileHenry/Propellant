use foundry::{create_entity, create_entities, AsAny, Updatable, System};
use propellant::*;





fn main() {

    let mut engine = PropellantEngine::default()
        .with_window().unwrap();    

    let _cam = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., 3., -4.)),
        Camera::main(800., 450., 0.1, 100., 1.5)
    );
    let _cubes = create_entities!(engine.world_mut(); 10_000,
        |i| Transform::origin().translated(glam::vec3(0., 0., -(i as f32))),
        |i| MeshRendererBuilder::new(Mesh::cube(1. / (i + 1) as f32), Material::default())
    );

    engine.world_mut().register_system(System::new(Box::new(FPSCounter{timer: 0., frames: 0}), foundry::UpdateFrequency::PerFrame), 11);

    engine.main_loop();
}

#[derive(AsAny)]
struct FPSCounter {
    timer: f32,
    frames: usize,
}

impl Updatable for FPSCounter {
    fn update(&mut self, _components: &mut foundry::ComponentTable, delta: f32) {
        self.frames += 1;
        self.timer += delta;
        if self.timer > 1. {
            println!("FPS: {}", self.frames);
            self.frames = 0;
            self.timer = 0.;
        }
    }
}
