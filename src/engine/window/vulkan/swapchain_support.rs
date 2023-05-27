use crate::engine::errors::PropellantError;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::HasBuilder;

#[derive(Clone, Debug)]
pub struct SwapchainSupport {
    capabilities: vulkanalia::vk::SurfaceCapabilitiesKHR,
    formats: Vec<vulkanalia::vk::SurfaceFormatKHR>,
    present_modes: Vec<vulkanalia::vk::PresentModeKHR>,
}

impl SwapchainSupport {
    pub fn get(
        instance: &vulkanalia::Instance,
        physical_device: vulkanalia::vk::PhysicalDevice,
        surface: vulkanalia::vk::SurfaceKHR
    ) -> Result<SwapchainSupport, PropellantError> {
        unsafe { Ok(SwapchainSupport {
            capabilities: instance.get_physical_device_surface_capabilities_khr(physical_device, surface)?,
            formats: instance.get_physical_device_surface_formats_khr(physical_device, surface)?,
            present_modes: instance.get_physical_device_surface_present_modes_khr(physical_device, surface)?,
        })}
    }

    pub fn is_sufficient(&self) -> bool {
        // check if the swap chain have a sufficient support for default propellant capabilities.
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }

    pub fn capabilities(&self) -> vulkanalia::vk::SurfaceCapabilitiesKHR {
        self.capabilities
    } 

    pub fn format(&self) -> vulkanalia::vk::SurfaceFormatKHR {
        self.formats
            .iter()
            .cloned()
            .find(|f| {
                f.format == vulkanalia::vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vulkanalia::vk::ColorSpaceKHR::SRGB_NONLINEAR
                }
            )
            .unwrap_or_else(|| self.formats[0])
    }

    pub fn present_mode(&self) -> vulkanalia::vk::PresentModeKHR {
        self.present_modes
            .iter()
            .cloned()
            .find(|m| *m == vulkanalia::vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vulkanalia::vk::PresentModeKHR::FIFO)
    }

    pub fn extent(&self, window: &winit::window::Window) -> vulkanalia::vk::Extent2D {
        if self.capabilities.current_extent.width != u32::max_value() {
            self.capabilities.current_extent
        } else {
            let size = window.inner_size();
            let clamp = |min: u32, max: u32, v: u32| min.max(max.min(v));
            vulkanalia::vk::Extent2D::builder()
                .width(clamp(
                    self.capabilities.min_image_extent.width,
                    self.capabilities.max_image_extent.width,
                    size.width,
                ))
                .height(clamp(
                    self.capabilities.min_image_extent.height,
                    self.capabilities.max_image_extent.height,
                    size.height,
                ))
                .build()
        }
    }
}