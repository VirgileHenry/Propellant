use foundry::{create_entity, Updatable, System, AsAny, component_iterator};
use propellant::*;





fn main() {

    let mut mesh_lib = MeshLibrary::new();
    mesh_lib.register_mesh(id("cube"), Mesh::cube(1.0));

    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_mesh_library(mesh_lib);
    

    let _cam = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., 1., -4.)),
        Camera::main(800., 450., 0.1, 100., 1.5)
    );
    let _cube = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(-1., 1., 0.)),
        MeshRenderer::new(id("cube"), Material::default())
    );
    let _cube = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(2., 0., 0.)),
        MeshRenderer::new(id("cube"), Material::default())
    );

    engine.world_mut().register_system(Rotater::new(), 11);


    engine.main_loop();
}

#[derive(AsAny)]
struct Rotater {}

impl Rotater {
    pub fn new() -> System {
        System::new(Box::new(Rotater{}), foundry::UpdateFrequency::PerFrame)
    }
}

impl Updatable for Rotater {
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        for (_entity, (tf, _mr)) in component_iterator!(components; mut Transform, MeshRenderer) {
            tf.rotate(glam::Quat::from_rotation_y(delta));
        }
    }
}
