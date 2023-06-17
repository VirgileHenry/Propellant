
/// Ask the renderer to rebuild the scene.
/// This flag should be inserted in the component table when the geometry changed.
pub struct RequireSceneRebuildFlag;


/// Tere are new meshes in the lib that need to be loaded.
#[derive(Debug, Clone, Copy)]
pub struct RequireResourcesLoadingFlag(u64);

#[allow(unused)]
impl RequireResourcesLoadingFlag {
    pub const MESHES: RequireResourcesLoadingFlag = RequireResourcesLoadingFlag(1 << 0);
    pub const TEXTURES: RequireResourcesLoadingFlag = RequireResourcesLoadingFlag(1 << 1);
    pub const ALL: RequireResourcesLoadingFlag = RequireResourcesLoadingFlag(u64::MAX);

    pub fn contains(&self, flag: RequireResourcesLoadingFlag) -> bool {
        self.0 & flag.0 != 0
    }
}

/// There are new data in the transfer manager that we should load to the gpu.
pub struct RequireMemoryTransfersFlag;