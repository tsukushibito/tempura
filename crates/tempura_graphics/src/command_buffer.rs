use std::rc::Rc;

use ash::vk;

use crate::command_pool::CommandPool;
use crate::graphics_device::GraphicsDevice;

pub struct CommandBuffer {
    graphics_device: Rc<GraphicsDevice>,
    command_pool: Rc<CommandPool>,
    command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
    pub(crate) fn new(
        graphics_device: &Rc<GraphicsDevice>,
        command_pool: &Rc<CommandPool>,
        command_buffer: vk::CommandBuffer,
    ) -> Self {
        Self {
            graphics_device: graphics_device.clone(),
            command_pool: command_pool.clone(),
            command_buffer,
        }
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.graphics_device
                .device()
                .free_command_buffers(self.command_pool.command_pool(), &[self.command_buffer]);
        }
    }
}
