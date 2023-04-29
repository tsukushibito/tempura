use std::rc::Rc;

use ash::vk;

use crate::CommandPool;
use crate::VulkanDevice;

pub struct CommandBuffer {
    vulkan_device: Rc<VulkanDevice>,
    command_pool: Rc<CommandPool>,
    command_buffer: vk::CommandBuffer,
}

impl CommandBuffer {
    pub(crate) fn new(
        vulkan_device: &Rc<VulkanDevice>,
        command_pool: &Rc<CommandPool>,
        command_buffer: vk::CommandBuffer,
    ) -> Self {
        Self {
            vulkan_device: vulkan_device.clone(),
            command_pool: command_pool.clone(),
            command_buffer,
        }
    }
}

impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe {
            self.vulkan_device
                .device()
                .free_command_buffers(self.command_pool.command_pool(), &[self.command_buffer]);
        }
    }
}
