use std::rc::Rc;

use ash::vk;

use crate::{Device, TvResult};

pub struct Semaphore {
    device: Rc<Device>,
    semaphore: vk::Semaphore,
}

impl Semaphore {
    pub(crate) fn new(device: &Rc<Device>) -> TvResult<Self> {
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
