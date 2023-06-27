use foundry::{create_entity, Updatable, System, AsAny};
use propellant::*;

use rand::Rng;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let mut resources = ProppellantResources::default();
    resources.meshes_mut().register_mesh(id("cube1"), Mesh::cube(0.01));
    resources.meshes_mut().register_mesh(id("cube2"), Mesh::cube(0.02));
    resources.meshes_mut().register_mesh(id("cube3"), Mesh::cube(0.03));
    resources.meshes_mut().register_mesh(id("cube4"), Mesh::cube(0.05));

    let mut engine = PropellantEngine::default()
        .with_window()?
        .with_resources(resources);
    
    // sun 
    engine.world_mut().add_singleton(DirectionnalLight::new(
        glam::vec3(1., 1., 1.),
        glam::vec3(1., 1., 1.),
        glam::vec3(-1., -1., -1.)
    ));
    let _cam = create_entity!(engine.world_mut();
        Transform::origin().translated(glam::vec3(0., 1., -4.)),
        Camera::main_perspective(800., 450., 0.1, 100., 1.5)
    );

    engine.world_mut().register_system(ObjectLoader::new(), 11);
    engine.world_mut().register_system(System::new(Box::new(FPSCounter{timer: -3., frames: 0}), foundry::UpdateFrequency::PerFrame), 12);


    engine.main_loop();

    Ok(())
}

#[derive(AsAny)]
struct ObjectLoader {
    counter: u64,
}

impl ObjectLoader {
    pub fn new() -> System {
        System::new(Box::new(ObjectLoader{counter: 0}), foundry::UpdateFrequency::PerFrame)
    }
}

impl Updatable for ObjectLoader {
    fn update(&mut self, components: &mut foundry::ComponentTable, _delta: f32) {
        // create a new texture every frame
        self.counter += 1;
        let offset = glam::vec3(
            (rand::thread_rng().gen::<f32>() - 0.5) * 10.,
            (rand::thread_rng().gen::<f32>() - 0.5) * 10.,
            (rand::thread_rng().gen::<f32>() - 0.5) * 10.,
        );
        let _entity = match self.counter % 4 {
            0 => create_entity!(components;
                Transform::origin().translated(offset),
                MeshRenderer::new(
                    id("cube1"),
                    Material::default().with_prop(PhongMaterialProperties::default().colored(glam::vec3(0.6, 0., 0.)))
                )
            ),
            1 => create_entity!(components;
                Transform::origin().translated(offset),
                MeshRenderer::new(
                    id("cube2"),
                    Material::default().with_prop(PhongMaterialProperties::default().colored(glam::vec3(0.6, 0.6, 0.)))
                )
            ),
            2 => create_entity!(components;
                Transform::origin().translated(offset),
                MeshRenderer::new(
                    id("cube3"),
                    Material::default().with_prop(PhongMaterialProperties::default().colored(glam::vec3(0.6, 0., 0.6)))
                )
            ),
            _ => create_entity!(components;
                Transform::origin().translated(offset),
                MeshRenderer::new(
                    id("cube4"),
                    Material::default().with_prop(PhongMaterialProperties::default().colored(glam::vec3(0., 0.6, 0.6)))
                )
            ),
        };
        components.add_singleton(RequireSceneRebuildFlag);
    }
}



#[derive(AsAny)]
struct FPSCounter {
    timer: f32,
    frames: usize,
}

impl Updatable for FPSCounter {
    fn update(&mut self, _components: &mut foundry::ComponentTable, delta: f32) {
        self.timer += delta;
        if self.timer > 0. {
            self.frames += 1;
        }
        if self.timer > 1. {
            println!("{} FPS - (frame time: {}ms)", self.frames, 1000. / self.frames as f32);
            self.frames = 0;
            self.timer = 0.;
        }
    }
}
