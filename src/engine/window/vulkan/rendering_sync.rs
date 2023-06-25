use vulkanalia::vk::HasBuilder;
use vulkanalia::vk::Handle;
use vulkanalia::vk::DeviceV1_0;

use crate::engine::errors::PResult;

/// Wraps the vulkan sync objects in a struct, with all the methods to wait.
/// This struct will manage the images in flight, the semaphores and the fences.
pub struct RenderingSync<const MAX_FRAMES_IN_FLIGHT: usize> {
    pub image_available: [vulkanalia::vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    pub render_finished: [vulkanalia::vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    pub frames_in_flight: [vulkanalia::vk::Fence; MAX_FRAMES_IN_FLIGHT],
    pub images_in_flight: Vec<vulkanalia::vk::Fence>,
    pub current_frame: usize,
}

impl<const MAX_FRAMES_IN_FLIGHT: usize> RenderingSync<MAX_FRAMES_IN_FLIGHT> {
    pub fn create(
        vk_device: &vulkanalia::Device,
        swapchain_images_count: usize,
    ) -> PResult<RenderingSync<MAX_FRAMES_IN_FLIGHT>> {
        // create the sync objects
        let mut image_available = [vulkanalia::vk::Semaphore::null(); MAX_FRAMES_IN_FLIGHT];
        let mut render_finished = [vulkanalia::vk::Semaphore::null(); MAX_FRAMES_IN_FLIGHT];
        let mut frames_in_flight = [vulkanalia::vk::Fence::null(); MAX_FRAMES_IN_FLIGHT];
        // create the info for the sync objects
        let semaphore_info = vulkanalia::vk::SemaphoreCreateInfo::builder();
        let fence_info = vulkanalia::vk::FenceCreateInfo::builder()
            .flags(vulkanalia::vk::FenceCreateFlags::SIGNALED); //by default, they are signaled so we won't wait on the first frame.

        // init all the sync objects
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            unsafe {
                image_available[i] = vk_device.create_semaphore(&semaphore_info, None)?;
                render_finished[i] = vk_device.create_semaphore(&semaphore_info, None)?;
                frames_in_flight[i] = vk_device.create_fence(&fence_info, None)?;
            }
        }

        let images_in_flight = (0..swapchain_images_count).map(|_| vulkanalia::vk::Fence::null()).collect::<Vec<_>>();
        
        Ok(RenderingSync {
            image_available,
            render_finished,
            frames_in_flight,
            images_in_flight,
            current_frame: 0,
        })
    }
    
    pub fn image_available_semaphore(&self) -> vulkanalia::vk::Semaphore {
        self.image_available[self.current_frame]
    }

    pub fn render_finished_semaphore(&self) -> vulkanalia::vk::Semaphore {
        self.render_finished[self.current_frame]
    }

    pub fn frame_in_flight_fence(&self) -> vulkanalia::vk::Fence {
        self.frames_in_flight[self.current_frame]
    }

    pub fn wait_for_frame_flight_fence(&mut self, vk_device: &vulkanalia::Device) -> PResult<()> {
        unsafe {
            // wait for the frame on this fence to finish
            vk_device.wait_for_fences(
                &[self.frames_in_flight[self.current_frame]],
                true,
                u64::max_value(),
            )?;
        }
        Ok(())
    }

    pub fn wait_for_in_flight_image(&mut self, image_index: usize, vk_device: &vulkanalia::Device) -> PResult<()> {
        unsafe {
            // wait for any in flight image
            if !self.images_in_flight[image_index as usize].is_null() {
                vk_device.wait_for_fences(
                    &[self.images_in_flight[image_index as usize]],
                    true,
                    u64::max_value(),
                )?;
            }
        }
        // use the fence of the current frame as the fence for the image in flight
        self.images_in_flight[image_index as usize] = self.frames_in_flight[self.current_frame];
        
        Ok(())
    }

    pub fn reset_in_flight_frame_fence(&self, vk_device: &vulkanalia::Device) -> PResult<()> {
        unsafe {
            vk_device.reset_fences(&[self.frames_in_flight[self.current_frame]])?;
        }
        Ok(())
    }

    pub fn advance_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn resize_images_in_flight(&mut self, new_length: usize) {
        self.images_in_flight.resize(new_length, vulkanalia::vk::Fence::null());
    }

    pub unsafe fn destroy(&mut self, vk_device: &vulkanalia::Device) {
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            vk_device.destroy_semaphore(self.image_available[i], None);
            vk_device.destroy_semaphore(self.render_finished[i], None);
            vk_device.destroy_fence(self.frames_in_flight[i], None);
        }
        // do not clean up the images in flight fences, as they are copies of the frames in flight fences.
    }
}

