/// Trait for any propellant flags.
/// Flags should no store more data than a u64.
/// this trait allow flag to u64 conversion and vice versa, accross any flags.
pub trait PropellantFlag {
    fn flag(&self) -> u64;
    fn from_flag(flag: u64) -> Self;
}
/// Ask the renderer to rebuild the scene.
/// This flag should be inserted in the component table when the geometry changed.
pub struct RequireSceneRebuildFlag;

impl PropellantFlag for RequireSceneRebuildFlag {
    fn flag(&self) -> u64 {
        0
    }
    fn from_flag(_: u64) -> Self {
        RequireSceneRebuildFlag
    }
}

/// Tere are new meshes in the lib that need to be loaded.
#[derive(Debug, Clone, Copy)]
pub struct RequireResourcesLoadingFlag(u64);

impl RequireResourcesLoadingFlag {
    pub const MESHES: RequireResourcesLoadingFlag = RequireResourcesLoadingFlag(1 << 0);
    pub const TEXTURES: RequireResourcesLoadingFlag = RequireResourcesLoadingFlag(1 << 1);
    pub const ALL: RequireResourcesLoadingFlag = RequireResourcesLoadingFlag(u64::MAX);

    pub fn contains(&self, flag: RequireResourcesLoadingFlag) -> bool {
        self.0 & flag.0 != 0
    }
}

impl PropellantFlag for RequireResourcesLoadingFlag {
    fn flag(&self) -> u64 {
        self.0
    }
    fn from_flag(flag: u64) -> Self {
        RequireResourcesLoadingFlag(flag)
    }
}


pub struct RequireCommandBufferRebuildFlag;

impl PropellantFlag for RequireCommandBufferRebuildFlag {
    fn flag(&self) -> u64 {
        0
    }
    fn from_flag(_: u64) -> Self {
        RequireCommandBufferRebuildFlag
    }
}
