use std::rc::Rc;

use ash::vk;

use crate::CommandBuffer;
use crate::Result;
use crate::VulkanDevice;

pub enum QueueFamily {
    Graphics,
    Present,
}

pub struct CommandPool {
    vulkan_device: Rc<VulkanDevice>,
    command_pool: vk::CommandPool,
}

impl CommandPool {
    pub(crate) fn new(vulkan_device: &Rc<VulkanDevice>, queue_family_index: u32) -> Result<Self> {
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            vulkan_device
                .device()
                .create_command_pool(&command_pool_create_info, None)?
        };

        Ok(Self {
            vulkan_device: vulkan_device.clone(),
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
    ) -> Result<Vec<Rc<CommandBuffer>>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(level)
            .command_buffer_count(command_buffer_count);

        let command_buffers = unsafe {
            self.vulkan_device
                .device()
                .allocate_command_buffers(&command_buffer_allocate_info)?
        };

        let command_buffers = command_buffers
            .iter()
            .map(|&command_buffer| {
                Rc::new(CommandBuffer::new(
                    &self.vulkan_device,
                    self,
                    command_buffer,
                ))
            })
            .collect::<Vec<Rc<CommandBuffer>>>();

        Ok(command_buffers)
    }

    pub fn reset(&self) -> Result<()> {
        unsafe {
            self.vulkan_device
                .device()
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())?;
        }

        Ok(())
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe { self.vulkan_device.device().device_wait_idle().unwrap() };
        unsafe {
            self.vulkan_device
                .device()
                .destroy_command_pool(self.command_pool, None)
        };
    }
}
