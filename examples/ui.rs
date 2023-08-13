use foundry::create_entity;
use glam::Vec3;
use propellant::*;


fn main() {

    let resources = PropellantResources::default();

    let inputs = InputHandlerBuilder::empty()
        .with_starting_ui_context();

    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_ui_resolution(1.2).unwrap()
        .with_input_handler(inputs).unwrap()
        .with_resources(resources).unwrap();

    /*
    let _top_left = create_entity!(
        engine.world_mut();
        UiTransform::new(
            glam::vec2(-5., 10.),
            glam::vec2(0.5, 0.),
            glam::vec2(300., 60.),
            glam::vec2(0., 0.),
            UiAnchor::TopRight,
            0,
        ),
        UiMaterial::colored(Vec3::new(1.0, 0.5, 0.8), 20.).to_mesh_renderer()
    );
    let _top_right = create_entity!(
        engine.world_mut();
        UiTransform::new(
            glam::vec2(5., 10.),
            glam::vec2(0.5, 0.),
            glam::vec2(300., 60.),
            glam::vec2(0., 0.),
            UiAnchor::TopLeft,
            0,
        ),
        UiMaterial::colored(Vec3::new(1.0, 0.8, 0.5), 20.).to_mesh_renderer()
    );
    */
    let bottom_tf = UiTransform::new(
        glam::vec2(0., -10.),
        glam::vec2(0.5, 1.),
        glam::vec2(-20., 120.),
        glam::vec2(1.0, 0.),
        UiAnchor::Bottom,
        0,
    );
    let sub_bottom_tf = UiTransform::new(
        glam::vec2(5., 0.),
        glam::vec2(0., 0.5),
        glam::vec2(100., -10.),
        glam::vec2(0., 1.),
        UiAnchor::Left,
        1,
    ); //.child_of(Some(&bottom_tf));
    let _bottom = create_entity!(
        engine.world_mut();
        bottom_tf,
        UiMaterial::colored(Vec3::new(0.5, 1.0, 0.8), 20.).to_mesh_renderer()
    );
    let _sub_bottom = create_entity!(
        engine.world_mut();
        sub_bottom_tf,
        UiMaterial::colored(Vec3::new(0.8, 0.8, 0.8), 20.).to_mesh_renderer()
    );
    /*
    let _side_1 = create_entity!(
        engine.world_mut();
        UiTransform::new(
            glam::vec2(10., 0.),
            glam::vec2(0., 0.5),
            glam::vec2(100., 60.),
            glam::vec2(0.0, 0.),
            UiAnchor::Left,
            0,
        ),
        UiMaterial::colored(Vec3::new(0.5, 0.8, 1.0), 20.).to_mesh_renderer()
    );
    let _side_2 = create_entity!(
        engine.world_mut();
        UiTransform::new(
            glam::vec2(10., 70.),
            glam::vec2(0., 0.5),
            glam::vec2(100., 60.),
            glam::vec2(0.0, 0.),
            UiAnchor::Left,
            0,
        ),
        UiMaterial::colored(Vec3::new(0.8, 0.5, 1.0), 20.).to_mesh_renderer()
    );
    let _side_2 = create_entity!(
        engine.world_mut();
        UiTransform::new(
            glam::vec2(10., -70.),
            glam::vec2(0., 0.5),
            glam::vec2(100., 60.),
            glam::vec2(0.0, 0.),
            UiAnchor::Left,
            0,
        ),
        UiMaterial::colored(Vec3::new(0.8, 1.0, 0.5), 20.).to_mesh_renderer()
    );
    */

    // engine.world_mut().register_system(UiManager::new(), id("ui_manager"));

    engine.main_loop();
}
