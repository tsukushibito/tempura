use std::rc::Rc;

use ash::vk;

use crate::VulkanDevice;

pub struct Semaphore {
    vulkan_device: Rc<VulkanDevice>,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub(crate) fn new(
        vulkan_device: &Rc<VulkanDevice>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();

        let semaphore = unsafe {
            vulkan_device
                .device()
                .create_semaphore(&semaphore_create_info, None)?
        };

        Ok(Self {
            vulkan_device: vulkan_device.clone(),
            semaphore,
        })
    }

    pub(crate) fn semaphore(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_device.device().device_wait_idle().unwrap();
        }
        unsafe {
            self.vulkan_device
                .device()
                .destroy_semaphore(self.semaphore, None);
        }
    }
}
