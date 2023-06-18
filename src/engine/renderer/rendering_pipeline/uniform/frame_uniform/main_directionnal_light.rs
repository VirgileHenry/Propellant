use foundry::ComponentTable;

use crate::engine::{
    lights::directionnal_light::DirectionnalLight,
    consts::PROPELLANT_DEBUG_FEATURES
};

use super::AsPerFrameUniform;

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct MainDirectionnalLight {
    main_light: DirectionnalLight,
    padding: f32, 
}

impl AsPerFrameUniform for MainDirectionnalLight {
    fn get_uniform(components: &ComponentTable) -> Self {
        match components.get_singleton::<DirectionnalLight>() {
            Some(main_light) => MainDirectionnalLight {
                main_light: main_light.clone(),
                padding: Default::default(),
            },
            None => {
                if PROPELLANT_DEBUG_FEATURES {
                    println!("[PROPELLANT DEBUG] No main light found.");
                }
                MainDirectionnalLight {
                    main_light: DirectionnalLight::black(),
                    padding: Default::default(),
                }
            },
        }
    }
}