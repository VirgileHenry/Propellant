use crate::{PropellantEngine, PropellantResources};
use self::resource_loading::RequireResourcesLoadingFlag;

use super::errors::PResult;

pub(crate) mod resource_loading;

#[derive(Debug, Clone, Copy)]
pub enum PropellantFlag {
    #[cfg(feature = "window")]
    RequireSceneRebuild,
    #[cfg(feature = "window")]
    RequireCommandBufferRebuild,
    #[cfg(feature = "resources")]
    RequireResourcesLoading(RequireResourcesLoadingFlag),
    UiRequireScreenSize,
}

impl PropellantEngine {
    pub fn handle_flag(&mut self, flag: PropellantFlag) -> PResult<()> {
        match flag {
            #[cfg(feature = "window")]
            PropellantFlag::RequireSceneRebuild => self.window.renderer_mut().request_scene_rebuild(),
            #[cfg(feature = "window")]
            PropellantFlag::RequireCommandBufferRebuild => self.window.renderer_mut().request_command_buffer_rebuild(),
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
                    },
                    None => println!("[PROPELLANT DEBUG] Resources loading requested, but no resources found."),
                }
            },
            _ => {},
        }

        Ok(())
    }
}