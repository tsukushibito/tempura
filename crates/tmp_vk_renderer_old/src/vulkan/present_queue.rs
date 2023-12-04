use std::rc::Rc;

use ash::vk;

use super::{device::Device, semaphore::Semaphore, swapchain::Swapchain};
use crate::TmpResult;

pub struct PresentQueue {
    device: Rc<Device>,
    queue: vk::Queue,
}

impl PresentQueue {
    pub fn new(device: &Rc<Device>, queue: vk::Queue) -> Self {
        Self {
            device: device.clone(),
            queue,
        }
    }

    pub fn handle(&self) -> vk::Queue {
        self.queue
    }

    pub fn present(
        &self,
        swapchain: &Swapchain,
        image_index: u32,
        wait_semaphores: &[&Semaphore],
    ) -> TmpResult<()> {
        let wait_semaphores = wait_semaphores
            .iter()
            .map(|s| s.handle())
            .collect::<Vec<vk::Semaphore>>();

        let swapchains = [swapchain.handle()];
        let image_indices = [image_index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&wait_semaphores)
            .swapchains(&swapchains)
            .image_indices(&image_indices)
            .build();

        unsafe {
            self.device
                .swapchain_loader()
                .queue_present(self.queue, &present_info)?
        };

        Ok(())
    }
}
