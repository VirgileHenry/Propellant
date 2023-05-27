use crate::engine::errors::PropellantError;
use vulkanalia::vk::{InstanceV1_0, KhrSurfaceExtension};

/// This represent the index of a the queue family that we will be using.
#[derive(Copy, Clone, Debug)]
pub struct QueueFamilyIndices(u32);

// todo : separate queue for buffer transfers, to improve perfs.
// https://kylemayes.github.io/vulkanalia/vertex/staging_buffer.html

impl QueueFamilyIndices {
    /// Finds a queue family that meet our needs, and return it's index under the form of a queue family.
    /// It can be done to look for different indices for graphics or presentation, but it does not change a lot and adds overhead.
    pub unsafe fn get(
        instance: &vulkanalia::Instance,
        physical_device: vulkanalia::vk::PhysicalDevice,
        surface: vulkanalia::vk::SurfaceKHR,
    ) -> Result<QueueFamilyIndices, PropellantError> {
        let properties = instance
            .get_physical_device_queue_family_properties(physical_device);

        for (index, properties) in properties.iter().enumerate() {
            let index = match u32::try_from(index) {
                Ok(n) => n,
                Err(_) => return Err(PropellantError::NoFittingVulkanDevice),
            };
            // all our requiremenets here
            if
                properties.queue_flags.contains(vulkanalia::vk::QueueFlags::GRAPHICS) &&
                instance.get_physical_device_surface_support_khr(physical_device, index, surface)?
            {
                return Ok(QueueFamilyIndices(index))
            }
        }

        Err(PropellantError::NoFittingVulkanDevice)
    }

    pub fn index(&self) -> u32 {
        self.0
    }
}