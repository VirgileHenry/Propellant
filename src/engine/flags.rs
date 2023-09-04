use crate::{PropellantEngine, PropellantResources, UiTextRenderer};
use self::resource_loading::RequireResourcesLoadingFlag;

use super::{errors::PResult, ui::{ui_resolution::UiResolution, ui_transform::UiTransform}, consts::PROPELLANT_DEBUG_FEATURES};

pub(crate) mod resource_loading;

#[derive(Debug, Clone, Copy)]
pub enum PropellantFlag {
    #[cfg(feature = "window")]
    /// Tell the renderer the scene have been invalidated and neeeds rebuilt.
    RequireSceneRebuild,
    #[cfg(feature = "resources")]
    /// We added resourcesin the resource lib that need to be loaded.
    RequireResourcesLoading(RequireResourcesLoadingFlag),
    #[cfg(feature = "ui")]
    /// The ui elements need to know what the screen resolution is.
    UiRequireResolution,
}

impl PropellantEngine {
    pub fn handle_flag(&mut self, flag: PropellantFlag) -> PResult<()> {
        match flag {
            #[cfg(feature = "window")]
            PropellantFlag::RequireSceneRebuild => self.window.renderer_mut().request_scene_rebuild(),
            #[cfg(all(feature = "resources", feature = "window"))]
            PropellantFlag::RequireResourcesLoading(flags) => {
                let (world, window) = self.world_and_window_mut();
                match world.get_singleton_mut::<PropellantResources>() {
                    Some(resources) => {
                        let vk_interface = window.vk_interface_mut();
                        resources.load_resources(
                            flags,
                            &vk_interface.instance,
                            &vk_interface.device,
                            vk_interface.physical_device,
                            &mut vk_interface.transfer_manager,
                        )?;
                        vk_interface.check_and_process_memory_transfers()?;
                        // todo : maybe check textures where changed ?
                        window.renderer_mut().request_textures_reload();
                    },
                    None => println!("[PROPELLANT DEBUG] Resources loading requested, but no resources found."),
                }
            },
            #[cfg(feature = "ui")]
            PropellantFlag::UiRequireResolution => {
                let resolution = match self.world.get_singleton::<UiResolution>() {
                    Some(res) => *res,
                    None => {
                        if PROPELLANT_DEBUG_FEATURES {
                            println!("[PROPELLANT DEBUG] [UI] Ui require screen resolution flag set, but no existing screen resolution.");
                        }
                        UiResolution::default()
                    }
                };
                for (_, ui_tf) in self.world.query1d_mut::<UiTransform>() {
                    ui_tf.set_ui_resolution(resolution);
                }
                // temp: rebuild text here
                let fonts = self.world.get_singleton::<PropellantResources>().unwrap().fonts().clone();
                let ui_res = self.world.get_singleton::<UiResolution>().unwrap().clone();

                for (_, tf, tr) in self.world.query2d_mut::<UiTransform, UiTextRenderer>() {
                    let font = fonts.font(tr.font()).unwrap();
                    tr.rebuild_text(tf, &font, ui_res);
                }
            }
        }

        Ok(())
    }
}