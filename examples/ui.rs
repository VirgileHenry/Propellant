use foundry::create_entity;
use glam::Vec3;
use propellant::*;





fn main() {

    let resources = ProppellantResources::default();

    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_ui_resolution(1.5).unwrap()
        .with_resources(resources);

    let _top_left = create_entity!(
        engine.world_mut();
        Transform::ui_new(
            glam::vec2(-5., 10.),
            glam::vec2(0.5, 0.),
            glam::vec2(300., 60.),
            glam::vec2(0., 0.),
            UiAnchor::TopRight,
        ),
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(0.5, 1.0, 0.8), 20.))
    );
    let _top_right = create_entity!(
        engine.world_mut();
        Transform::ui_new(
            glam::vec2(5., 10.),
            glam::vec2(0.5, 0.),
            glam::vec2(300., 60.),
            glam::vec2(0., 0.),
            UiAnchor::TopLeft,
        ),
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(1.0, 0.5, 0.8), 20.))
    );
    let _bottom = create_entity!(
        engine.world_mut();
        Transform::ui_new(
            glam::vec2(0., -10.),
            glam::vec2(0.5, 1.),
            glam::vec2(-20., 60.),
            glam::vec2(1.0, 0.),
            UiAnchor::Bottom,
        ),
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(1.0, 0.8, 0.5), 20.))
    );
    let _side_1 = create_entity!(
        engine.world_mut();
        Transform::ui_new(
            glam::vec2(10., 0.),
            glam::vec2(0., 0.5),
            glam::vec2(100., 60.),
            glam::vec2(0.0, 0.),
            UiAnchor::Left,
        ),
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(0.5, 0.8, 1.0), 20.))
    );
    let _side_2 = create_entity!(
        engine.world_mut();
        Transform::ui_new(
            glam::vec2(10., 70.),
            glam::vec2(0., 0.5),
            glam::vec2(100., 60.),
            glam::vec2(0.0, 0.),
            UiAnchor::Left,
        ),
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(0.8, 0.5, 1.0), 20.))
    );
    let _side_2 = create_entity!(
        engine.world_mut();
        Transform::ui_new(
            glam::vec2(10., -70.),
            glam::vec2(0., 0.5),
            glam::vec2(100., 60.),
            glam::vec2(0.0, 0.),
            UiAnchor::Left,
        ),
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(0.8, 1.0, 0.5), 20.))
    );

    engine.main_loop();
}

