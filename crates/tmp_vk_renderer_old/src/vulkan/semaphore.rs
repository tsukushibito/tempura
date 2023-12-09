use std::rc::Rc;

use ash::vk;

use super::device::Device;
use crate::TmpResult;

pub struct Semaphore {
    device: Rc<Device>,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub fn new(device: &Rc<Device>) -> TmpResult<Self> {
        let semaphore_create_info = vk::SemaphoreCreateInfo::builder().build();

        let semaphore = unsafe {
            device
                .handle()
                .create_semaphore(&semaphore_create_info, None)?
        };

        Ok(Self {
            device: device.clone(),
            semaphore,
        })
    }

    pub fn handle(&self) -> vk::Semaphore {
        self.semaphore
    }
}

impl Drop for Semaphore {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().device_wait_idle().unwrap();
        }
        unsafe {
            self.device.handle().destroy_semaphore(self.semaphore, None);
        }
    }
}