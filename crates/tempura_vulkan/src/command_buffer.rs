use std::rc::Rc;

use ash::vk;

use crate::CommandPool;
use crate::Device;

pub struct CommandBuffer {
    device: Rc<Device>,
    command_pool: Rc<CommandPool>,
    command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
    pub(crate) fn new(
        device: &Rc<Device>,
        command_pool: &Rc<CommandPool>,
        command_buffer: vk::CommandBuffer,
    ) -> Self {
        Self {
            device: device.clone(),
            command_pool: command_pool.clone(),
            command_buffer,
        }
    }

    pub(crate) fn handle(&self) -> vk::CommandBuffer {
        self.command_buffer
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle()
                .free_command_buffers(self.command_pool.handle(), &[self.command_buffer]);
        }
    }
}
