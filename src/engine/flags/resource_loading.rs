
#[derive(Debug, Copy, Clone)]
pub struct RequireResourcesLoadingFlag(u64);

impl RequireResourcesLoadingFlag {
    pub const MESHES: Self = Self(1 << 0);
    pub const TEXTURES: Self = Self(1 << 1);
    pub const ALL: Self = Self(u64::MAX);
    
    pub fn empty() -> Self {
        Self(0)
    }

    pub fn contains(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }
}

impl std::ops::BitOr for RequireResourcesLoadingFlag {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}