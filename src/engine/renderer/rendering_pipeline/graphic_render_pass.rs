use std::collections::HashMap;

use crate::ProppellantResources;
use crate::engine::renderer::graphics_pipeline::graphics_pipeline_builder::GraphicsPipelineBuilder;
use crate::engine::{renderer::graphics_pipeline::GraphicsPipeline, errors::PResult, window::vulkan::swapchain_interface::SwapchainInterface};

use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::DeviceV1_0;

use super::intermediate_render_targets::IntermediateRenderTarget;

enum RenderingPipelinePassTarget {
    /// We are targetting the swapchain, and only own the framebuffers.
    /// The image and views are owned by the swapchain.
    Swapchain(Vec<vulkanalia::vk::Framebuffer>),
    /// We are targetting an intermediate render target, and own the image and views.
    Intermediate(IntermediateRenderTarget),
}

impl RenderingPipelinePassTarget {
    pub fn framebuffer(&self, image_index: usize) -> vulkanalia::vk::Framebuffer {
        match self {
            RenderingPipelinePassTarget::Swapchain(framebuffers) => framebuffers[image_index],
            RenderingPipelinePassTarget::Intermediate(framebuffers) => framebuffers.framebuffer(),
        }
    }
}

pub struct GraphicRenderpass {
    /// The pipelines for this pass.
    /// todo : add abstraction to handle both graphics and compute pipelines.
    pipelines: HashMap<u64, GraphicsPipeline>,
    /// target framebuffers for this pass.
    /// These are the framebuffers of the swapchain if this is the final pass.
    target: RenderingPipelinePassTarget,
    /// renderpass object.
    render_pass: vulkanalia::vk::RenderPass,
}

impl GraphicRenderpass {


    pub fn create_final_pass(
        pipelines: &mut HashMap<u64, GraphicsPipelineBuilder>,
        vk_device: &vulkanalia::Device,
        swapchain: &SwapchainInterface,
    ) -> PResult<GraphicRenderpass> {
        // build the render pass and the framebuffers, targetting the swapchain images
        let renderpass = Self::create_final_render_pass(
            vk_device,
            swapchain.format()
        )?;
        let framebuffers = Self::create_final_framebuffers(
            vk_device,
            swapchain.image_views(),
            renderpass,
            swapchain.extent()
        )?;
        // build the pipelines
        let pipelines = pipelines.drain().map(|(id, pipeline)| {
            // create the pipeline hash map for this layer.
            pipeline.build(
                vk_device,
                swapchain.extent(),
                swapchain.images().len(),
                renderpass
            ).and_then(|result| Ok((id, result)))
        }).collect::<PResult<HashMap<u64, GraphicsPipeline>>>()?;

        Ok(GraphicRenderpass {
            pipelines,
            target: RenderingPipelinePassTarget::Swapchain(framebuffers),
            render_pass: renderpass,
        })

    }

    pub fn register_draw_commands(
        &self,
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        swapchain_extent: vulkanalia::vk::Extent2D,
        resources: &ProppellantResources,
        image_index: usize,
    ) -> PResult<()> {

        // final render pass
        let render_area = vulkanalia::vk::Rect2D::builder()
            .offset(vulkanalia::vk::Offset2D::default())
            .extent(swapchain_extent);

        let color_clear_value = vulkanalia::vk::ClearValue {
            color: vulkanalia::vk::ClearColorValue {
                float32: [0., 0., 0., 1.0],
            },
        };

        let clear_values = &[color_clear_value];

        let info = vulkanalia::vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass)
            .framebuffer(self.target.framebuffer(image_index))
            .render_area(render_area)
            .clear_values(clear_values);
        
        unsafe { vk_device.cmd_begin_render_pass(command_buffer, &info, vulkanalia::vk::SubpassContents::INLINE) };
        
        // for each pipeline
        for (_, pipeline) in self.pipelines.iter() {
            pipeline.register_draw_commands(
                vk_device,
                image_index,
                command_buffer,
                resources,
            );
        }
        unsafe { vk_device.cmd_end_render_pass(command_buffer) };

        Ok(())
    }

    pub fn recreation_cleanup(
        &mut self,
        vk_device: &vulkanalia::Device
    ) {
        self.pipelines.values_mut().for_each(|pipeline| pipeline.recreation_cleanup(vk_device));
        match self.target {
            RenderingPipelinePassTarget::Swapchain(ref mut framebuffers) => {
                for framebuffer in framebuffers.drain(..) {
                    unsafe { vk_device.destroy_framebuffer(framebuffer, None) };
                }
            },
            RenderingPipelinePassTarget::Intermediate(ref mut intermediate) => {
                intermediate.destroy(vk_device);
            }
        }
        unsafe { vk_device.destroy_render_pass(self.render_pass, None) };
    }

    pub fn recreate(
        &mut self,
        vk_device: &vulkanalia::Device,
        swapchain: &SwapchainInterface,
    ) -> PResult<()> {
        match self.target {
            RenderingPipelinePassTarget::Swapchain(ref mut framebuffers) => {
                self.render_pass = Self::create_final_render_pass(
                    vk_device,
                    swapchain.format(),
                )?;
                framebuffers.append(&mut Self::create_final_framebuffers(
                    vk_device,
                    swapchain.image_views(),
                    self.render_pass,
                    swapchain.extent(),
                )?);
            },
            RenderingPipelinePassTarget::Intermediate(ref mut _intermediate) => {
                unimplemented!()
            }
        }
        for pipeline in self.pipelines.values_mut() {
            pipeline.recreate(
                vk_device,
                swapchain.extent(),
                self.render_pass,
            )?;
        }
        Ok(())
    }
    
    fn create_final_render_pass(
        device: &vulkanalia::Device,
        swapchain_format: vulkanalia::vk::Format
    ) -> PResult<vulkanalia::vk::RenderPass> {
        // create the color attachment
        let color_attachment = vulkanalia::vk::AttachmentDescription::builder()
            .format(swapchain_format)
            .samples(vulkanalia::vk::SampleCountFlags::_1)
            .load_op(vulkanalia::vk::AttachmentLoadOp::CLEAR)
            .store_op(vulkanalia::vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vulkanalia::vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vulkanalia::vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vulkanalia::vk::ImageLayout::UNDEFINED)
            .final_layout(vulkanalia::vk::ImageLayout::PRESENT_SRC_KHR);
        // create the color attachment reference
        let color_attachment_ref = vulkanalia::vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vulkanalia::vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        // create the subpass
        let color_attachments = &[color_attachment_ref];
        let subpass = vulkanalia::vk::SubpassDescription::builder()
            .pipeline_bind_point(vulkanalia::vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(color_attachments);
        // create the subpass dependency
        let dependency = vulkanalia::vk::SubpassDependency::builder()
            .src_subpass(vulkanalia::vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vulkanalia::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vulkanalia::vk::AccessFlags::empty())
            .dst_stage_mask(vulkanalia::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vulkanalia::vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
        // create the render pass
        let attachments = &[color_attachment];
        let subpasses = &[subpass];
        let dependencies = &[dependency];
        let info = vulkanalia::vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies);
        
        Ok(unsafe {
            device.create_render_pass(&info, None)?
        })
    }

    fn create_final_framebuffers(
        device: &vulkanalia::Device,
        image_views: &Vec<vulkanalia::vk::ImageView>,
        render_pass: vulkanalia::vk::RenderPass,
        extent: vulkanalia::vk::Extent2D
    ) -> PResult<Vec<vulkanalia::vk::Framebuffer>> {
        Ok(image_views
            .iter()
            .map(|i| {
                let attachments = &[*i];
                let create_info = vulkanalia::vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(attachments)
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1);

                unsafe {
                    device.create_framebuffer(&create_info, None)
                }
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn pipelines(&self) -> &HashMap<u64, GraphicsPipeline> {
        &self.pipelines
    }

    pub fn pipelines_mut(&mut self) -> &mut HashMap<u64, GraphicsPipeline> {
        &mut self.pipelines
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device
    ) {
        for (_id, mut pipeline) in self.pipelines.drain() {
            pipeline.destroy(vk_device);
        }
        unsafe {
            vk_device.destroy_render_pass(self.render_pass, None);
        }
        match &mut self.target {
            RenderingPipelinePassTarget::Swapchain(framebuffers) => {
                for framebuffer in framebuffers {
                    unsafe {
                        vk_device.destroy_framebuffer(*framebuffer, None);
                    }
                }
            },
            RenderingPipelinePassTarget::Intermediate(intermediate_render_target) => {
                intermediate_render_target.destroy(vk_device);
            }
        }
    }
}

pub trait RenderPass {

}