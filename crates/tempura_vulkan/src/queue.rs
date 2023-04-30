use std::rc::Rc;

use ash::vk;

use crate::{CommandBuffer, Fence, Semaphore, Swapchain, TvResult, VulkanDevice, Window};

pub struct Queue {
    vulkan_device: Rc<VulkanDevice>,
    queue: vk::Queue,
}

impl Queue {
    pub(crate) fn new(vulkan_device: &Rc<VulkanDevice>, queue: vk::Queue) -> Self {
        Self {
            vulkan_device: vulkan_device.clone(),
            queue,
        }
    }

    pub(crate) fn queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn submit(
        &self,
        command_buffers: &[&CommandBuffer],
        wait_semaphores: &[&Semaphore],
        wait_stages: &[vk::PipelineStageFlags],
        signal_semaphores: &[&Semaphore],
        fence: Option<&Fence>,
    ) -> TvResult<()> {
        let command_buffers = command_buffers
            .iter()
            .map(|cb| cb.command_buffer())
            .collect::<Vec<vk::CommandBuffer>>();

        let wait_semaphores = wait_semaphores
            .iter()
            .map(|s| s.semaphore())
            .collect::<Vec<vk::Semaphore>>();

        let signal_semaphores = signal_semaphores
            .iter()
            .map(|s| s.semaphore())
            .collect::<Vec<vk::Semaphore>>();

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .build();

        let fence = if let Some(f) = fence {
            f.fence()
        } else {
            vk::Fence::null()
        };

        unsafe {
            self.vulkan_device
                .device()
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
