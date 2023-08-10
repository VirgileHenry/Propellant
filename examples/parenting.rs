use foundry::{create_entity, Updatable, System, AsAny};
use propellant::*;


fn main() {

    let mut resources = ProppellantResources::default();
    resources.meshes_mut().register_mesh(id("cube"), Mesh::cube(1.0));

    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_resources(resources);
    

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
    let cube1_tf = Transform::origin().translated(glam::vec3(-1., 1., 0.));
    let cube2_tf = Transform::origin().translated(glam::vec3(2., 0., 0.)).child_of(Some(&cube1_tf));
    let _cube = create_entity!(engine.world_mut();
        cube1_tf,
        RotatingObject,
        InstancedMeshRenderer::new(
            id("cube"),
            PhongMaterial::default().colored(glam::vec3(0.6, 0., 0.))
        )
    );
    let _cube = create_entity!(engine.world_mut();
        cube2_tf,
        InstancedMeshRenderer::new(
            id("cube"),
            PhongMaterial::default().colored(glam::vec3(0., 0.6, 0.))
        )
    );

    engine.world_mut().register_system(Rotater::new(), 11);


    engine.main_loop();
}

struct RotatingObject;

#[derive(AsAny)]
struct Rotater {}

impl Rotater {
    pub fn new() -> System {
        System::new(Rotater{}, foundry::UpdateFrequency::PerFrame)
    }
}

impl Updatable for Rotater {
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        for (_, tf, _) in components.query2d_mut::<Transform, RotatingObject>() {
            tf.rotate(glam::Quat::from_rotation_y(delta));
        }
    }
}
