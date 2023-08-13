use foundry::{create_entity, Updatable, System, AsAny};
use propellant::*;





fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut resources = PropellantResources::default();
    resources.meshes_mut().register_mesh(id("quad"), Mesh::flat_quad(2.0));
    resources.meshes_mut().register_mesh(id("cube"), Mesh::cube(0.6));

    let mut engine = PropellantEngine::default()
        .with_window()?
        .with_resources(resources).unwrap();
    

    let _cam = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., 1., -4.)),
        Camera::main_perspective(800., 450., 0.1, 100., 1.5)
    );

    engine.world_mut().register_system(TextureLoader::new(), 11);


    engine.main_loop();

    Ok(())
}

#[derive(AsAny)]
struct TextureLoader {
    counter: u64,
}

impl TextureLoader {
    pub fn new() -> System {
        System::new(TextureLoader{counter: 0}, foundry::UpdateFrequency::PerFrame)
    }
}

impl Updatable for TextureLoader {
    fn update(&mut self, components: &mut foundry::ComponentTable, _delta: f32) {
        // create a new texture every frame
        let texture_lib = match components.get_singleton_mut::<PropellantResources>() {
            Some(resources) => resources.textures_mut(),
            None => {
                println!("No texture library found!");
                return;
            }
        };
        self.counter += 1;
        match texture_lib.register_texture(self.counter, include_bytes!("model/texture.jpg")) {
            Ok(_) => println!("Registered texture {}", self.counter),
            Err(e) => println!("Failed to register texture {}: {}", self.counter, e),
        };
        // components.add_singleton(RequireResourcesLoadingFlag::TEXTURES);
    }
}
