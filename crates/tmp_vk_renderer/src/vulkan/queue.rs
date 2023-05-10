use std::rc::Rc;

use ash::vk;

use super::{command_buffer::CommandBuffer, device::Device, fence::Fence, semaphore::Semaphore};
use crate::TmpResult;

pub struct Queue {
    device: Rc<Device>,
    queue: vk::Queue,
}

impl Queue {
    pub fn new(device: &Rc<Device>, queue: vk::Queue) -> Self {
        Self {
            device: device.clone(),
            queue,
        }
    }

    pub fn handle(&self) -> vk::Queue {
        self.queue
    }

    pub fn submit(
        &self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        signal_semaphores: &[&Semaphore],
        fence: Option<&Fence>,
    ) -> TmpResult<()> {
        let command_buffers = command_buffers
            .iter()
            .map(|cb| cb.handle())
            .collect::<Vec<vk::CommandBuffer>>();

        let wait_semaphores = wait_semaphores
            .iter()
            .map(|s| s.handle())
            .collect::<Vec<vk::Semaphore>>();

        let signal_semaphores = signal_semaphores
            .iter()
            .map(|s| s.handle())
            .collect::<Vec<vk::Semaphore>>();

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();

        let fence = if let Some(f) = fence {
            f.handle()
        } else {
            vk::Fence::null()
        };

        unsafe {
            self.device
                .handle()
                .queue_submit(self.queue, &[submit_info], fence)?;
        }

        Ok(())
    }

    // pub fn present<T: Window>(
    //     &self,
    //     swapchain: &Swapchain<T>,
    //     image_index: u32,
    //     wait_semaphores: &[&Semaphore],
    // ) -> TvResult<()> {
    //     let present_info = vk::PresentInfoKHR::builder()
    //         .wait_semaphores(wait_semaphores)
    //         .swapchains(&[swapchain])
    //         .image_indices(&[image_index])
    //         .build();

    //     unsafe {
    //         self.vulkan_device
    //             .device()
    //             .queue_present_khr(self.queue, &present_info)?;
    //     }

    //     Ok(())
    // }
}
