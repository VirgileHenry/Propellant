use crate::engine::errors::PResult;
use crate::engine::window::vulkan::vulkan_image::VulkanImage;

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;

use super::attachments::depth_attachment::create_depth_objects;


pub struct FinalRenderTargetBuilder {

}

impl Default for FinalRenderTargetBuilder {
    fn default() -> Self {
        Self {

        }
    }
}


pub struct FinalRenderTarget {
    /// Framebuffers that reference the images of the swapchain, used to display to the screen.
    framebuffers: Vec<vulkanalia::vk::Framebuffer>,
    /// Any addional images that are also used in the final render pass.
    /// These does not include the swapchain images.
    additional_images: Vec<VulkanImage>,
    /// Views of the additional images.
    additional_image_views: Vec<vulkanalia::vk::ImageView>,
}

impl FinalRenderTarget {
    pub fn create(
        _builder: FinalRenderTargetBuilder,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        image_views: &Vec<vulkanalia::vk::ImageView>,
        render_pass: vulkanalia::vk::RenderPass,
        extent: vulkanalia::vk::Extent2D,
    ) -> PResult<FinalRenderTarget> {

        // hard coded depth objects for now
        let (depth_image, depth_image_view) = create_depth_objects(
            vk_instance,
            vk_device,
            vk_physical_device,
            extent,
        )?;

        Ok(FinalRenderTarget {
            framebuffers: Self::create_framebuffers(
                vk_device,
                image_views,
                depth_image_view,
                render_pass,
                extent
            )?,
            additional_images: vec![depth_image],
            additional_image_views: vec![depth_image_view],
        })
    }

    fn create_framebuffers(
        vk_device: &vulkanalia::Device,
        image_views: &Vec<vulkanalia::vk::ImageView>,
        depth_image_view: vulkanalia::vk::ImageView,
        render_pass: vulkanalia::vk::RenderPass,
        extent: vulkanalia::vk::Extent2D
    ) -> PResult<Vec<vulkanalia::vk::Framebuffer>> {
        Ok(image_views
            .iter()
            .map(|i| {
                let attachments = &[*i, depth_image_view];
                let create_info = vulkanalia::vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);

                unsafe {
                    vk_device.create_framebuffer(&create_info, None)
                }
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn recreation_cleanup(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        for framebuffer in self.framebuffers.drain(..) {
            unsafe {
                vk_device.destroy_framebuffer(framebuffer, None);
            }
        }
        for image_view in self.additional_image_views.drain(..) {
            unsafe {
                vk_device.destroy_image_view(image_view, None);
            }
        }
        for mut image in self.additional_images.drain(..) {
            image.destroy(vk_device);
        }
    }

    pub fn recreate(
        &mut self,
        vk_instance: &vulkanalia::Instance,
        vk_device: &vulkanalia::Device,
        vk_physical_device: vulkanalia::vk::PhysicalDevice,
        image_views: &Vec<vulkanalia::vk::ImageView>,
        render_pass: vulkanalia::vk::RenderPass,
        extent: vulkanalia::vk::Extent2D,
    ) -> PResult<()> {
        // hard coded depth objects for now
        let (depth_image, depth_image_view) = create_depth_objects(
            vk_instance,
            vk_device,
            vk_physical_device,
            extent,
        )?;

        self.framebuffers = Self::create_framebuffers(
            vk_device,
            image_views,
            depth_image_view, // todo : hard coded for now
            render_pass,
            extent
        )?;

        self.additional_images.push(depth_image);
        self.additional_image_views.push(depth_image_view);


        Ok(())
    }

    pub fn framebuffer(
        &self,
        image_index: usize,
    ) -> vulkanalia::vk::Framebuffer {
        self.framebuffers[image_index]
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        for framebuffer in self.framebuffers.iter_mut() {
            unsafe {
                vk_device.destroy_framebuffer(*framebuffer, None);
            }
        }
        for image_view in self.additional_image_views.iter_mut() {
            unsafe {
                vk_device.destroy_image_view(*image_view, None);
            }
        }
        for image in self.additional_images.iter_mut() {
            image.destroy(vk_device);
        }
    }
}