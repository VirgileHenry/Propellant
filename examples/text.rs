use foundry::*;
use propellant::*;


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut resources = PropellantResources::default();
    
    resources.meshes_mut().register_mesh(id("quad"), MeshType::flat_quad(10.0));
    resources.meshes_mut().register_mesh(id("cat"), MeshType::load_static_mesh(include_bytes!("model/cat.gmesh"))?);
    
    let cat_texture_index = resources.textures_mut().register_texture(id("cat"), include_bytes!("model/cat_texture.png"))?;
    let font_atlas_index = resources.load_font(id("font"), include_bytes!("text/noto-serif.ttf"))?;

    let inputs = InputHandler::builder()
        .with_starting_ui_context();

    let mut engine = PropellantEngine::builder()
        .with_input_handler(inputs)
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
        InstancedMeshRenderer::<PhongMaterial, StaticMesh>::new(
            id("quad"),
            PhongMaterial::default().textured(font_atlas_index)
        )
    );
    let _cat = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., -1., 0.)),
        InstancedMeshRenderer::<PhongMaterial, StaticMesh>::new(
            id("cat"),
            PhongMaterial::default().colored(glam::vec3(1., 1., 1.)).textured(cat_texture_index)
        )
    );

    engine.world_mut().register_system(Rotater::new(), 11);

    let _text_renderer = create_entity!(
        engine.world_mut();
        UiTransform::new(
            glam::vec2(20., 20.),
            glam::vec2(0., 0.),
            glam::vec2(0., 0.),
            glam::vec2(1., 1.),
            UiAnchor::TopLeft,
            0,
        ),
        UiTextRenderer::new("Hello World !\nThis is an example text.\nEnjoy this demo :)".to_string(), font_atlas_index, glam::vec3(1., 0., 1.))
    );
    
    // engine.world_mut().register_system(UiManager::new(), id("ui_manager"));

    engine.main_loop()?;

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
        for (_entity, tf, _mr) in components.query2d_mut::<Transform, InstancedMeshRenderer<PhongMaterial, StaticMesh>>() {
            tf.rotate(glam::Quat::from_rotation_y(delta * 0.1));
        }
    }
}
