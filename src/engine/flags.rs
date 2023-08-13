use crate::{PropellantEngine, PropellantResources, PropellantWindow, id};

use self::resource_loading::RequireResourcesLoadingFlag;
use super::errors::PResult;


pub(crate) mod resource_loading;

#[derive(Debug, Clone, Copy)]
pub enum PropellantFlag {
    RequireSceneRebuild,
    RequireResourcesLoading(RequireResourcesLoadingFlag),
    RequireCommandBufferRebuild,
    UiRequireScreenSize,
}

impl PropellantEngine {
    pub fn handle_engine_flag(&mut self, flag: PropellantFlag) -> PResult<()> {
        match flag {
            // reload resources
            PropellantFlag::RequireResourcesLoading(resource_flags) => match self.world.get_system_and_world_mut(id("window")) {
                Some((system, world)) => match (system.try_get_updatable_mut::<PropellantWindow>(), world.get_singleton_mut::<PropellantResources>()) {
                    (Some(window), Some(resource_lib)) => {
                        // load meshes
                        let vk_interface = window.vk_interface_mut();
                        resource_lib.load_resources(
                            resource_flags,
                            &vk_interface.instance,
                            &vk_interface.device,
                            vk_interface.physical_device,
                            &mut vk_interface.transfer_manager,
                        )?;
                        // rebuild the resources uniforms.
                        // todo : rebuild frame uniform, it should be in the components?
                    }
                    _ => {}
                },
                _ => {}
            },
            // flags are sent to the renderer
            PropellantFlag::RequireSceneRebuild |
            PropellantFlag::RequireCommandBufferRebuild => match self.world.get_system_mut(id("window")) {
                Some(system) => match system.try_get_updatable_mut::<PropellantWindow>() {
                    Some(window) => window.renderer_mut().engine_flag(flag),
                    _ => {}
                },
                _ => {}
            },
            // the ui requires the screen size
            PropellantFlag::UiRequireScreenSize => match self.world.get_system_and_world_mut(id("window")) {
                Some((system, world)) => match system.try_get_updatable::<PropellantWindow>() {
                    Some(window) => {

                    },
                    _ => {}
                },
                _ => {}
            },
        }

        Ok(())
    }

}