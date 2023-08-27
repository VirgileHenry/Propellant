use std::collections::VecDeque;


/// Enum to ease Vulkan sync mechanism, especially when ressources are recreated.
#[derive(Debug)]
pub enum VulkanSyncState<T> {
    /// The object is currently working
    Sane(T),
    /// The object have been recreated and is waiting for the old one to finish.
    Syncing(VecDeque<T>),
}

impl<T> VulkanSyncState<T> {
    /// Creates a new Vulkan sync object
    pub fn new(t: T) -> Self {
        VulkanSyncState::Sane(t)
    }

}