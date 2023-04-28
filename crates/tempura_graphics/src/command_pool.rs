use std::rc::Rc;

use ash::vk;

use crate::command_buffer::CommandBuffer;
use crate::graphics_device::GraphicsDevice;

pub enum QueueFamily {
    Graphics,
    Present,
}

pub struct CommandPool {
    graphics_device: Rc<GraphicsDevice>,
    command_pool: vk::CommandPool,
}

impl CommandPool {
    pub(crate) fn new(
        graphics_device: &Rc<GraphicsDevice>,
        queue_family_index: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            graphics_device
                .device()
                .create_command_pool(&command_pool_create_info, None)?
        };

        Ok(Self {
            graphics_device: graphics_device.clone(),
            command_pool,
        })
    }

    pub(crate) fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }

    pub fn allocate_command_buffers(
        self: &Rc<Self>,
        level: vk::CommandBufferLevel,
        command_buffer_count: u32,
    ) -> Result<Vec<Rc<CommandBuffer>>, Box<dyn std::error::Error>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(level)
            .command_buffer_count(command_buffer_count);

        let command_buffers = unsafe {
            self.graphics_device
                .device()
                .allocate_command_buffers(&command_buffer_allocate_info)?
        };

        let command_buffers = command_buffers
            .iter()
            .map(|&command_buffer| {
                Rc::new(CommandBuffer::new(
                    &self.graphics_device,
                    self,
                    command_buffer,
                ))
            })
            .collect::<Vec<Rc<CommandBuffer>>>();

        Ok(command_buffers)
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe { self.graphics_device.device().device_wait_idle().unwrap() };
        unsafe {
            self.graphics_device
                .device()
                .destroy_command_pool(self.command_pool, None)
        };
    }
}
