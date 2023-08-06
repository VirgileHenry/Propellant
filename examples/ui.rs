use foundry::{create_entity, System, ComponentTable, Updatable, AsAny};
use glam::Vec3;
use propellant::*;





fn main() {

    let resources = ProppellantResources::default();

    let inputs = InputHandlerBuilder::empty()
        .with_starting_ui_context();

    let mut engine = PropellantEngine::default()
        .with_window().unwrap()
        .with_ui_resolution(1.2).unwrap()
        .with_input_handler(inputs).unwrap()
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
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(0.5, 0.8, 1.0), 20.)),
        UiEventListener::new(UiExpand::new(
            Transform::ui_new(
                glam::vec2(10., 0.),
                glam::vec2(0., 0.5),
                glam::vec2(100., 60.),
                glam::vec2(0.0, 0.),
                UiAnchor::Left,
            ).world_pos(),
            Transform::ui_new(
                glam::vec2(10., 0.),
                glam::vec2(0., 0.5),
                glam::vec2(200., 60.),
                glam::vec2(0.0, 0.),
                UiAnchor::Left,
            ).world_pos(),
            5., Box::new(|x| 1. - (1. - x) * (1. - x)),
        ))
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
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(0.8, 0.5, 1.0), 20.)),
        UiEventListener::new(UiExpand::new(
            Transform::ui_new(
                glam::vec2(10., 70.),
                glam::vec2(0., 0.5),
                glam::vec2(100., 60.),
                glam::vec2(0.0, 0.),
                UiAnchor::Left,
            ).world_pos(),
            Transform::ui_new(
                glam::vec2(10., 70.),
                glam::vec2(0., 0.5),
                glam::vec2(200., 60.),
                glam::vec2(0.0, 0.),
                UiAnchor::Left,
            ).world_pos(),
            5., Box::new(|x| 1. - (1. - x) * (1. - x)),
        ))
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
        MeshRenderer::ui_renderer(UiMaterial::colored(Vec3::new(0.8, 1.0, 0.5), 20.)),
        UiEventListener::new(UiExpand::new(
            Transform::ui_new(
                glam::vec2(10., -70.),
                glam::vec2(0., 0.5),
                glam::vec2(100., 60.),
                glam::vec2(0.0, 0.),
                UiAnchor::Left,
            ).world_pos(),
            Transform::ui_new(
                glam::vec2(10., -70.),
                glam::vec2(0., 0.5),
                glam::vec2(200., 60.),
                glam::vec2(0.0, 0.),
                UiAnchor::Left,
            ).world_pos(),
            5., Box::new(|x| 1. - (1. - x) * (1. - x)),
        ))
    );

    engine.world_mut().register_system(UiManager::new(), id("ui_manager"));

    engine.main_loop();
}

#[derive(Debug)]
enum UiExpandstate {
    Retracted,
    Expanded,
    Retracting(f32),
    Expanding(f32),
}

struct UiExpand {
    state: UiExpandstate,
    retracted_state: glam::Mat4,
    expanded_state: glam::Mat4,
    anim_speed: f32,
    anim_curve: Box<dyn Fn(f32) -> f32>,
}

impl UiExpand {
    pub fn new(retracted_state: glam::Mat4, expanded_state: glam::Mat4, anim_speed: f32, anim_curve: Box<dyn Fn(f32) -> f32>) -> UiExpand {
        UiExpand {
            state: UiExpandstate::Retracted,
            retracted_state,
            expanded_state,
            anim_speed,
            anim_curve,
        }
    }
}

impl UiListenerCallback for UiExpand {
    fn on_event(&mut self, event: UiEvent, transform: &mut Transform) -> Option<Box<dyn Fn(&mut ComponentTable)>> {
        match event {
            UiEvent::MouseMove(cursor) => match (&self.state, transform.ui_contains_cursor(cursor)) {
                (UiExpandstate::Retracted, true) => self.state = UiExpandstate::Expanding(0.),
                (UiExpandstate::Retracting(v), true) => self.state = UiExpandstate::Expanding(*v),
                (UiExpandstate::Expanded, false) => self.state = UiExpandstate::Retracting(0.),
                (UiExpandstate::Expanding(v), false) => self.state = UiExpandstate::Retracting(*v),
                _ => {},
            },
            _ => {},
        }
        None
    }

    fn update(&mut self, transform: &mut Transform, delta: f32) {
        match &mut self.state {
            UiExpandstate::Retracting(animation_time) => {
                *animation_time += delta * self.anim_speed;
                let anim_value = (self.anim_curve)(*animation_time);
                let tf_mat = if *animation_time >= 1. {
                    self.state = UiExpandstate::Retracted;
                    self.retracted_state
                }
                else {
                    self.retracted_state * anim_value + self.expanded_state * (1. - anim_value)
                };
                unsafe { transform.set_world_matrix(tf_mat) };
            },
            UiExpandstate::Expanding(animation_time) => {
                *animation_time += delta * self.anim_speed;
                let anim_value = (self.anim_curve)(*animation_time);
                let tf_mat = if *animation_time >= 1. {
                    self.state = UiExpandstate::Expanded;
                    self.expanded_state
                }
                else {
                    self.expanded_state * anim_value + self.retracted_state * (1. - anim_value)
                };
                unsafe { transform.set_world_matrix(tf_mat) };
            },
            _ => {},
        }
    }
}

#[derive(AsAny)]
struct UiManager {}

impl UiManager{
    pub fn new() -> System {
        System::new(
            UiManager {},
            foundry::UpdateFrequency::PerFrame,
        )
    }
}

impl Updatable for UiManager {
    fn update(&mut self, components: &mut ComponentTable, delta: f32) {
        for (_entity, transform, listener) in components.query2d_mut::<Transform, UiEventListener>() {
            match listener.listener() {
                Some(callback) => callback.update(transform, delta),
                None => (),
            }
        }
    }
}