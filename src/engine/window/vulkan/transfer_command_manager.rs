use crate::engine::consts::PROPELLANT_DEBUG_FEATURES;
use crate::engine::errors::PResult;
use crate::engine::errors::PropellantError;
use crate::engine::errors::loading_errors::LoadingError;

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::Handle;

use super::vulkan_buffer::VulkanBuffer;

pub enum TransferCommand {
    /// staging buffer, destination buffer, size
    CopyBuffer(VulkanBuffer, vulkanalia::vk::Buffer, u64),
    /// staging buffer, destination image, width, height
    CopyImage(VulkanBuffer, vulkanalia::vk::Image, u32, u32),
    /// Transition the image to the given layout.
    TransitionImageLayout(vulkanalia::vk::Image, vulkanalia::vk::Format, vulkanalia::vk::ImageLayout, vulkanalia::vk::ImageLayout),
}

impl TransferCommand {

    pub fn buffer_transfer(
        staging: VulkanBuffer,
        destination: vulkanalia::vk::Buffer,
        size: u64,
    ) -> TransferCommand {
        TransferCommand::CopyBuffer(staging, destination, size)
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device
    ) {
        match self {
            TransferCommand::CopyBuffer(staging_buffer, _, _) => staging_buffer.destroy(vk_device),
            TransferCommand::CopyImage(staging_buffer, _, _, _) => staging_buffer.destroy(vk_device),
            TransferCommand::TransitionImageLayout(_, _, _, _) => {}
        }
    }
}

pub struct TransferCommandManager {
    command_pool: vulkanalia::vk::CommandPool,
    transfer_queue: Vec<TransferCommand>,
    transfer_fences: Vec<vulkanalia::vk::Fence>,
}

impl TransferCommandManager {
    /// Creates a new command pool and buffers.
    pub fn create(
        vk_device: &vulkanalia::Device,
        indices: super::queues::QueueFamilyIndices,
    ) -> PResult<TransferCommandManager> {
        // create the frame buffers
        let info = vulkanalia::vk::CommandPoolCreateInfo::builder()
            .queue_family_index(indices.index())
            .flags(vulkanalia::vk::CommandPoolCreateFlags::empty());

        let command_pool = unsafe {vk_device.create_command_pool(&info, None)?};

        Ok(TransferCommandManager {
            command_pool,
            transfer_queue: Vec::new(),
            transfer_fences: Vec::new(),
        })
    }

    /// tells wether there are waiting transfers or not.
    pub fn need_transfers(&self) -> bool {
        !self.transfer_queue.is_empty()
    }

    /// Proccess all the required transfers.
    pub fn transfer(
        &mut self,
        vk_device: &vulkanalia::Device,
        queue: vulkanalia::vk::Queue,
    ) -> PResult<()> {
        
        let info = vulkanalia::vk::CommandBufferAllocateInfo::builder()
            .level(vulkanalia::vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.command_pool)
            .command_buffer_count(1);

        let command_buffer = unsafe { vk_device.allocate_command_buffers(&info)?[0] };

        let info = vulkanalia::vk::CommandBufferBeginInfo::builder()
            .flags(vulkanalia::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        unsafe { vk_device.begin_command_buffer(command_buffer, &info)? };

        for transfer in self.transfer_queue.iter() {
            match transfer {
                TransferCommand::CopyBuffer(staging, destination, size) => Self::record_buffer_transfer(vk_device, command_buffer, staging, *destination, *size)?,
                TransferCommand::CopyImage(staging, destination, width, height) => Self::record_image_transfer(vk_device, command_buffer, staging, *destination, *width, *height)?,
                TransferCommand::TransitionImageLayout(image, format, old_layout, new_layout) => Self::record_pipeline_barrier(vk_device, command_buffer, *image, *format,  *old_layout, *new_layout)?,
            }
        }

        unsafe { vk_device.end_command_buffer(command_buffer)? };

        // execute all registered commands.
        let command_buffers = &[command_buffer];
        let info = vulkanalia::vk::SubmitInfo::builder().command_buffers(command_buffers);            
        
        unsafe {
            vk_device.queue_submit(queue, &[info], vulkanalia::vk::Fence::null())?;
            vk_device.queue_wait_idle(queue)?;
        }
        

        // free the command buffers
        unsafe { vk_device.free_command_buffers(self.command_pool, command_buffers) };

        // finally, drain the queue (emptying it) and free the staging buffers.
        for mut transfer in self.transfer_queue.drain(..) {
            transfer.destroy(vk_device);
        }

        Ok(())
    }

    fn record_buffer_transfer(
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        staging: &VulkanBuffer,
        destination: vulkanalia::vk::Buffer,
        size: vulkanalia::vk::DeviceSize,
    ) -> PResult<()> {        

        let regions = vulkanalia::vk::BufferCopy::builder().size(size);
        unsafe { vk_device.cmd_copy_buffer(command_buffer, staging.buffer(), destination, &[regions]) };

        Ok(())
    }

    fn record_image_transfer(
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        staging: &VulkanBuffer,
        destination: vulkanalia::vk::Image,
        width: u32,
        height: u32,
    ) -> PResult<()> {

        // switch the image layout to transfer destination, using a barrier.
        Self::record_pipeline_barrier(vk_device, command_buffer, destination,
            vulkanalia::vk::Format::R8G8B8A8_SRGB,
            vulkanalia::vk::ImageLayout::UNDEFINED,
            vulkanalia::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;

        // copy the buffer to the image.
        let subresource = vulkanalia::vk::ImageSubresourceLayers::builder()
            .aspect_mask(vulkanalia::vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let region = vulkanalia::vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(subresource)
            .image_offset(vulkanalia::vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vulkanalia::vk::Extent3D { width, height, depth: 1 });

        unsafe {
            vk_device.cmd_copy_buffer_to_image(
                command_buffer,
                staging.buffer(),
                destination,
                vulkanalia::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        // switch the image layout to shader read.
        Self::record_pipeline_barrier(vk_device, command_buffer, destination,
            vulkanalia::vk::Format::R8G8B8A8_SRGB,
            vulkanalia::vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vulkanalia::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        Ok(())
    }


    fn record_pipeline_barrier(
        vk_device: &vulkanalia::Device,
        command_buffer: vulkanalia::vk::CommandBuffer,
        destination: vulkanalia::vk::Image,
        format: vulkanalia::vk::Format,
        old_layout: vulkanalia::vk::ImageLayout,
        new_layout: vulkanalia::vk::ImageLayout,
    ) -> PResult<()> {
        // create the access masks from the layouts.
        let (
            src_access_mask,
            dst_access_mask,
            src_stage_mask,
            dst_stage_mask,
        ) = match (old_layout, new_layout) {
            (vulkanalia::vk::ImageLayout::UNDEFINED, vulkanalia::vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                vulkanalia::vk::AccessFlags::empty(),
                vulkanalia::vk::AccessFlags::TRANSFER_WRITE,
                vulkanalia::vk::PipelineStageFlags::TOP_OF_PIPE,
                vulkanalia::vk::PipelineStageFlags::TRANSFER,
            ),
            (vulkanalia::vk::ImageLayout::TRANSFER_DST_OPTIMAL, vulkanalia::vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                vulkanalia::vk::AccessFlags::TRANSFER_WRITE,
                vulkanalia::vk::AccessFlags::SHADER_READ,
                vulkanalia::vk::PipelineStageFlags::TRANSFER,
                vulkanalia::vk::PipelineStageFlags::FRAGMENT_SHADER,
            ),
            (vulkanalia::vk::ImageLayout::UNDEFINED, vulkanalia::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
                vulkanalia::vk::AccessFlags::empty(),
                vulkanalia::vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vulkanalia::vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                vulkanalia::vk::PipelineStageFlags::TOP_OF_PIPE,
                vulkanalia::vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            ),
            _ => return Err(PropellantError::Loading(LoadingError::TextureLayoutTransitionMissing)),
        };

        let aspect_mask = if new_layout == vulkanalia::vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
            match format {
                vulkanalia::vk::Format::D32_SFLOAT_S8_UINT | vulkanalia::vk::Format::D24_UNORM_S8_UINT =>
                    vulkanalia::vk::ImageAspectFlags::DEPTH | vulkanalia::vk::ImageAspectFlags::STENCIL,
                _ => vulkanalia::vk::ImageAspectFlags::DEPTH
            }
        } else {
            vulkanalia::vk::ImageAspectFlags::COLOR
        };

        let subresource = vulkanalia::vk::ImageSubresourceRange::builder()
            .aspect_mask(aspect_mask)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let barrier = vulkanalia::vk::ImageMemoryBarrier::builder()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vulkanalia::vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vulkanalia::vk::QUEUE_FAMILY_IGNORED)
            .image(destination)
            .subresource_range(subresource)
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);

        unsafe {
            vk_device.cmd_pipeline_barrier(
                command_buffer,
                src_stage_mask,
                dst_stage_mask,
                vulkanalia::vk::DependencyFlags::empty(),
                &[] as &[vulkanalia::vk::MemoryBarrier],
                &[] as &[vulkanalia::vk::BufferMemoryBarrier],
                &[barrier],
            );
        }

        Ok(())
    }

    /// Add a transfer to do on the next transfer call.
    pub fn register_buffer_transfer(
        &mut self,
        vk_device: &vulkanalia::Device,
        staging: VulkanBuffer, // take ownership to destroy it when transfer is done.
        destination: vulkanalia::vk::Buffer,
        size: vulkanalia::vk::DeviceSize,
    ) -> PResult<()> {
        self.transfer_queue.push(TransferCommand::buffer_transfer(staging, destination, size));
        let fence_info = vulkanalia::vk::FenceCreateInfo::default();

        // complete the fence list.
        while self.transfer_fences.len() < self.transfer_queue.len() {
            self.transfer_fences.push(unsafe {
                vk_device.create_fence(&fence_info, None)?
            });
        }

        Ok(())
    }

    pub fn register_image_transfer(
        &mut self,
        vk_device: &vulkanalia::Device,
        staging: VulkanBuffer, // take ownership to destroy it when transfer is done.
        destination: vulkanalia::vk::Image,
        width: u32,
        height: u32,
    ) -> PResult<()> {
        self.transfer_queue.push(TransferCommand::CopyImage(staging, destination, width, height));
        let fence_info = vulkanalia::vk::FenceCreateInfo::default();

        // complete the fence list.
        while self.transfer_fences.len() < self.transfer_queue.len() {
            self.transfer_fences.push(unsafe {
                vk_device.create_fence(&fence_info, None)?
            });
        }

        Ok(())
    }

    pub fn register_transition_image_layout(
        &mut self,
        vk_device: &vulkanalia::Device,
        image: vulkanalia::vk::Image,
        format: vulkanalia::vk::Format,
        old_layout: vulkanalia::vk::ImageLayout,
        new_layout: vulkanalia::vk::ImageLayout,
    ) -> PResult<()> {
        self.transfer_queue.push(TransferCommand::TransitionImageLayout(image, format, old_layout, new_layout));
        let fence_info = vulkanalia::vk::FenceCreateInfo::default();

        // complete the fence list.
        while self.transfer_fences.len() < self.transfer_queue.len() {
            self.transfer_fences.push(unsafe {
                vk_device.create_fence(&fence_info, None)?
            });
        }

        Ok(())
    }

    pub fn destroy(
        &mut self,
        vk_device: &vulkanalia::Device,
    ) {
        for fence in self.transfer_fences.drain(..) {
            unsafe { vk_device.destroy_fence(fence, None) };
        }

        unsafe { vk_device.destroy_command_pool(self.command_pool, None) };

        // in debug mode, we set the command pool to null to mark it has been destroyed.
        if PROPELLANT_DEBUG_FEATURES {
            self.command_pool = vulkanalia::vk::CommandPool::null();
        }
    }
}

impl Drop for TransferCommandManager {
    fn drop(&mut self) {
        if PROPELLANT_DEBUG_FEATURES {
            // in debug mode, check the pool have indeed been destroyed.
            if self.command_pool != vulkanalia::vk::CommandPool::null() {
                println!("[PROPELLANT DEBUG] TransferCommandManager was not destroyed before being dropped.");
            }
        }
    }
}

