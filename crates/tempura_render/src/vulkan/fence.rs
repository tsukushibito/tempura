use std::rc::Rc;

use ash::vk;

use super::common::TvResult;
use super::device::Device;

pub struct Fence {
    device: Rc<Device>,
    fence: vk::Fence,
}

impl Fence {
    pub fn new(device: &Rc<Device>, signaled: bool) -> TvResult<Self> {
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(if signaled {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            })
            .build();

        let fence = unsafe { device.handle().create_fence(&fence_create_info, None)? };

        Ok(Self {
            device: device.clone(),
            fence,
        })
    }

    pub fn handle(&self) -> vk::Fence {
        self.fence
    }

    pub fn wait(&self) -> TvResult<()> {
        unsafe {
            self.device
                .handle()
                .wait_for_fences(&[self.fence], true, std::u64::MAX)?;
        }

        Ok(())
    }

    pub fn reset(&self) -> TvResult<()> {
        unsafe {
            self.device.handle().reset_fences(&[self.fence])?;
        }

        Ok(())
    }
}

impl Drop for Fence {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().device_wait_idle().unwrap();
        }
        unsafe {
            self.device.handle().destroy_fence(self.fence, None);
        }
    }
}
