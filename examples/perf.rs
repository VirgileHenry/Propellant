use foundry::{create_entity, create_entities, AsAny, Updatable, System};
use propellant::*;





fn main() {

    let mut mesh_lib = MeshLibrary::new();
    mesh_lib.register_mesh(id("cube"), Mesh::cube(1.));

    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_mesh_library(mesh_lib);

    let _cam = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., 3., -4.)),
        Camera::main(800., 450., 0.1, 100., 1.5)
    );
    let _cubes = create_entities!(engine.world_mut(); 1_000,
        |i| Transform::origin().translated(glam::vec3(0., 0., -(i as f32))).scaled(glam::vec3(1./ (i as f32 + 1.), 1./ (i as f32 + 1.), 1./ (i as f32 + 1.))),
        |_| MeshRenderer::new(id("cube"), Material::default())
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
