use foundry::{create_entity, Updatable, System, AsAny};
use propellant::*;





fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut resources = PropellantResources::default();
    
    resources.meshes_mut().register_mesh(id("quad"), Mesh::flat_quad(10.0));
    resources.meshes_mut().register_mesh(id("cat"), Mesh::from_bytes(include_bytes!("model/cat.gmesh"))?);
    
    let quad_texture_index = resources.textures_mut().register_texture(id("quad"), include_bytes!("model/texture.jpg"))?;
    let cat_texture_index = resources.textures_mut().register_texture(id("cat"), include_bytes!("model/cat_texture.png"))?;

    let mut engine = PropellantEngine::builder()
        .with_resources(resources);
    

    let _cam = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., -4., -8.)),
        Camera::main_perspective(450., 800., 0.1, 100., 1.5)
    );
    // sun 
    engine.world_mut().add_singleton(DirectionnalLight::new(
        glam::vec3(0.02, 0.02, 0.04),
        glam::vec3(0.8, 0.7, 0.75),
        glam::vec3(-1., -1., -1.).normalize(),
    ));
    let _quad = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., -1., 0.)),
        InstancedMeshRenderer::new(
            id("quad"),
            PhongMaterial::default().textured(quad_texture_index)
        )
    );
    let _cat = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., -1., 0.)),
        InstancedMeshRenderer::new(
            id("cat"),
            PhongMaterial::default().colored(glam::vec3(1., 1., 1.)).textured(cat_texture_index)
        )
    );

    engine.world_mut().register_system(Rotater::new(), 11);


    engine.main_loop().unwrap();

    Ok(())
}

#[derive(AsAny)]
struct Rotater {}

impl Rotater {
    pub fn new() -> System {
        System::new(Rotater{}, foundry::UpdateFrequency::PerFrame)
    }
}

impl Updatable for Rotater {
    fn update(&mut self, components: &mut foundry::ComponentTable, delta: f32) {
        for (_entity, tf, _mr) in components.query2d_mut::<Transform, InstancedMeshRenderer<PhongMaterial>>() {
            tf.rotate(glam::Quat::from_rotation_y(delta * 0.1));
        }
    }
}
