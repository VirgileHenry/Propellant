use vulkanalia::vk::PhysicalDeviceProperties;
use vulkanalia::vk::PhysicalDeviceFeatures;
use vulkanalia::vk::PhysicalDeviceType;


pub trait PhysicalDevicePreferences {
    fn is_device_compatible(&self, prop: PhysicalDeviceProperties, feat: PhysicalDeviceFeatures) -> bool;
    fn order_devices(&self, device1: (PhysicalDeviceProperties, PhysicalDeviceFeatures), device2: (PhysicalDeviceProperties, PhysicalDeviceFeatures)) -> std::cmp::Ordering;
}

pub struct DefaultPhysicalDevicePreferences;

impl PhysicalDevicePreferences for DefaultPhysicalDevicePreferences {
    fn is_device_compatible(&self, prop: PhysicalDeviceProperties, feat: PhysicalDeviceFeatures) -> bool {
        // allow default gpu types
        match prop.device_type {
            PhysicalDeviceType::CPU | 
            PhysicalDeviceType::VIRTUAL_GPU |
            PhysicalDeviceType::DISCRETE_GPU |
            PhysicalDeviceType::INTEGRATED_GPU => {},
            _ => return false,
        };
        // check default features
        // todo : neaded features, this is for example
        if feat.geometry_shader != vulkanalia::vk::TRUE {
            return false;
        }

        true
    }

    fn order_devices(&self, device1: (PhysicalDeviceProperties, PhysicalDeviceFeatures), device2: (PhysicalDeviceProperties, PhysicalDeviceFeatures)) -> std::cmp::Ordering {
        // first match on the types
        // todo !
        match (device1.0.device_type, device2.0.device_type) {
            (PhysicalDeviceType::CPU, PhysicalDeviceType::DISCRETE_GPU) => return std::cmp::Ordering::Greater,
            _ => {},
        }
    
    
        // could not sort them up, return equal
        std::cmp::Ordering::Equal
    }
}