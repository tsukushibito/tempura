use std::rc::Rc;

use ash::vk;

use crate::vulkan_device::VulkanDevice;

pub struct Fence {
    vulkan_device: Rc<VulkanDevice>,
    fence: vk::Fence,
}

impl Fence {
    pub(crate) fn new(
        vulkan_device: &Rc<VulkanDevice>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        let fence = unsafe {
            vulkan_device
                .device()
                .create_fence(&fence_create_info, None)?
        };

        Ok(Self {
            vulkan_device: vulkan_device.clone(),
            fence,
        })
    }

    pub(crate) fn fence(&self) -> vk::Fence {
        self.fence
    }

    pub fn wait(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.vulkan_device
                .device()
                .wait_for_fences(&[self.fence], true, std::u64::MAX)?;
        }

        Ok(())
    }

    pub fn reset(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            self.vulkan_device.device().reset_fences(&[self.fence])?;
        }

        Ok(())
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_device.device().device_wait_idle().unwrap();
        }
        unsafe {
            self.vulkan_device.device().destroy_fence(self.fence, None);
        }
    }
}