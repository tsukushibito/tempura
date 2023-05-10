use std::rc::Rc;

use ash::vk;

use super::command_buffer::CommandBuffer;
use super::common::TvResult;
use super::device::{Device, QueueFamily};

pub struct CommandPool {
    device: Rc<Device>,
    command_pool: vk::CommandPool,
}

impl CommandPool {
    pub fn new(device: &Rc<Device>, queue_family: QueueFamily) -> TvResult<Self> {
        let queue_family_index = device.queue_family_index_from_queue_family(queue_family);
        let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            device
                .handle()
                .create_command_pool(&command_pool_create_info, None)?
        };

        Ok(Self {
            device: device.clone(),
            command_pool,
        })
    }

    pub fn handle(&self) -> vk::CommandPool {
        self.command_pool
    }

    pub fn allocate_command_buffers(
        self: &Rc<Self>,
        level: vk::CommandBufferLevel,
        command_buffer_count: u32,
    ) -> TvResult<Vec<CommandBuffer>> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(self.command_pool)
            .level(level)
            .command_buffer_count(command_buffer_count);

        let command_buffers = unsafe {
            self.device
                .handle()
                .allocate_command_buffers(&command_buffer_allocate_info)?
        };

        let command_buffers = command_buffers
            .iter()
            .map(|&command_buffer| CommandBuffer::new(&self.device, self, command_buffer))
            .collect::<Vec<CommandBuffer>>();

        Ok(command_buffers)
    }

    pub fn reset(&self) -> TvResult<()> {
        unsafe {
            self.device
                .handle()
                .reset_command_pool(self.command_pool, vk::CommandPoolResetFlags::empty())?;
        }

        Ok(())
    }
}

impl Drop for CommandPool {
    fn drop(&mut self) {
        unsafe { self.device.handle().device_wait_idle().unwrap() };
        unsafe {
            self.device
                .handle()
                .destroy_command_pool(self.command_pool, None)
        };
    }
}
