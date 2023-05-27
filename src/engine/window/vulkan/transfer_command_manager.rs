use crate::engine::errors::PropellantError;

use vulkanalia::vk::DeviceV1_0;
use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::Handle;

use super::vulkan_buffer::VulkanBuffer;


pub struct TransferCommandManager {
    command_pool: vulkanalia::vk::CommandPool,
    transfer_queue: Vec<(VulkanBuffer, vulkanalia::vk::Buffer, u64)>,
    transfer_fences: Vec<vulkanalia::vk::Fence>,
}

impl TransferCommandManager {
    /// Creates a new command pool and buffers.
    pub fn create(
        vk_device: &vulkanalia::Device,
        indices: super::queues::QueueFamilyIndices,
    ) -> Result<TransferCommandManager, PropellantError> {
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
    ) -> Result<(), PropellantError> {
        
        let info = vulkanalia::vk::CommandBufferAllocateInfo::builder()
            .level(vulkanalia::vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.command_pool)
            .command_buffer_count(self.transfer_queue.len() as u32);

        let command_buffers = unsafe { vk_device.allocate_command_buffers(&info)? };

        for ((staging_buffer, dest_buffer, size), command_buffer) in self.transfer_queue.iter().zip(command_buffers.iter()) {
            let info = vulkanalia::vk::CommandBufferBeginInfo::builder()
                .flags(vulkanalia::vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

            unsafe { vk_device.begin_command_buffer(*command_buffer, &info)? };

            let regions = vulkanalia::vk::BufferCopy::builder().size(*size);
            unsafe { vk_device.cmd_copy_buffer(*command_buffer, staging_buffer.buffer(), *dest_buffer, &[regions]) };

            unsafe { vk_device.end_command_buffer(*command_buffer)? };

            let info = vulkanalia::vk::SubmitInfo::builder().command_buffers(command_buffers.as_slice());

            unsafe {
                vk_device.queue_submit(queue, &[info], vulkanalia::vk::Fence::null())?;
                vk_device.queue_wait_idle(queue)?;
            }
            
        }

        // free the command buffers
        unsafe { vk_device.free_command_buffers(self.command_pool, &command_buffers) };

        // finally, drain the queue (emptying it) and free the staging buffers.
        for (mut staging_buffer, _, _) in self.transfer_queue.drain(..) {
            staging_buffer.destroy(vk_device);
        }

        Ok(())
    }

    /// Add a transfer to do on the next transfer call.
    pub fn register_transfer(
        &mut self,
        vk_device: &vulkanalia::Device,
        staging: VulkanBuffer, // take ownership to destroy it when transfer is done.
        destination: vulkanalia::vk::Buffer,
        size: vulkanalia::vk::DeviceSize,
    ) -> Result<(), PropellantError> {
        self.transfer_queue.push((staging, destination, size));
        let fence_info = vulkanalia::vk::FenceCreateInfo::default();

        // complete the fence list.
        while self.transfer_fences.len() < self.transfer_queue.len() {
            self.transfer_fences.push(unsafe {
                vk_device.create_fence(&fence_info, None)?
            });
        }

        Ok(())
    }
}

