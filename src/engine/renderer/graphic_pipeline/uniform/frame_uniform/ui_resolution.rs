use crate::engine::consts::PROPELLANT_DEBUG_FEATURES;

use super::FrameUniform;


#[repr(C)]
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub struct UiResolution {
    pub resolution: f32,
    pub screen_width: f32,
    pub screen_height: f32,
}


impl FrameUniform for UiResolution {
    fn get_uniform(components: &foundry::ComponentTable) -> Self {
        match components.get_singleton::<UiResolution>() {
            Some(res) => *res,
            None => {
                if PROPELLANT_DEBUG_FEATURES {
                    println!("UiResolution singleton not found, creating a new one. This will imply false screen width and height.");
                }
                UiResolution {
                    resolution: 1.,
                    screen_width: 1.,
                    screen_height: 1.,
                }
            },
        }
    }
}