use std::fmt::Display;



pub enum DebugError {
    MissingVulkanDebugLayers,
    MissingResourceLibrary,
    NoMainCamera,
    NoMainLight,
}

impl Display for DebugError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugError::MissingVulkanDebugLayers => write!(f, "Missing Vulkan debug layers"),
            DebugError::MissingResourceLibrary => write!(f, "Missing resource library"),
            DebugError::NoMainCamera => write!(f, "No main camera"),
            DebugError::NoMainLight => write!(f, "No main light"),
        }
    }
}