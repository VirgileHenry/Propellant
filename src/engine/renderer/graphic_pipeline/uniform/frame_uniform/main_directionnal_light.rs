use foundry::ComponentTable;

use crate::engine::lights::directionnal_light::DirectionnalLight;

use super::FrameUniform;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct MainDirectionnalLight {
    direction: glam::Vec3,
    _padd_0: f32,
    ambiant_color: glam::Vec3,
    _padd_1: f32,
    direct_color: glam::Vec3,
    _padd_2: f32,
}

impl FrameUniform for MainDirectionnalLight {
    fn get_uniform(components: &ComponentTable) -> Self {
        match components.get_singleton::<DirectionnalLight>() {
            Some(main_light) => MainDirectionnalLight {
                direction: main_light.direction,
                _padd_0: 0.0,
                ambiant_color: main_light.ambiant_color,
                _padd_1: 0.0,
                direct_color: main_light.direct_color,
                _padd_2: 0.0
            },
            None => {
                MainDirectionnalLight {
                    direction: glam::Vec3::NEG_Y,
                    _padd_0: 0.0,
                    ambiant_color: glam::Vec3::ZERO,
                    _padd_1: 0.0,
                    direct_color: glam::Vec3::ZERO,
                    _padd_2: 0.0
                }
            },
        }
    }
}