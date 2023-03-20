use std::rc::Rc;

use ash::vk::{self, FenceCreateFlags};

use crate::{Device, VulkanObject};

pub struct FrameData {
    pub image_semaphore: vk::Semaphore,
    pub drawing_semaphore: vk::Semaphore,
    pub drawing_fence: vk::Fence,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,

    device: Rc<Device>,
}

impl FrameData {
    pub fn new(device: &Rc<Device>) -> Self {
        unsafe {
            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            let image_semaphore = device
                .device
                .create_semaphore(&semaphore_create_info, None)
                .expect("create_semaphore failed.");
            let drawing_semaphore = device
                .device
                .create_semaphore(&semaphore_create_info, None)
                .expect("create_semaphore failed.");

            let fence_create_info = vk::FenceCreateInfo::builder()
                .flags(FenceCreateFlags::SIGNALED)
                .build();
            let drawing_fence = device
                .device
                .create_fence(&fence_create_info, None)
                .expect("create fence error.");

            let command_pool_create_info = vk::CommandPoolCreateInfo::builder()
                .queue_family_index(device.graphics_queue_family_index)
                .build();
            let command_pool = device
                .device
                .create_command_pool(&command_pool_create_info, None)
                .expect("create command pool error.");

            let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(command_pool)
                .command_buffer_count(1)
                .level(vk::CommandBufferLevel::PRIMARY)
                .build();
            let command_buffers = device
                .device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("allocate command buffers error.");

            Self {
                image_semaphore,
                drawing_semaphore,
                drawing_fence,
                command_pool,
                command_buffers,
                device: device.clone(),
            }
        }
    }
}

impl Drop for FrameData {
    fn drop(&mut self) {
        self.device
            .push_dropped_object(VulkanObject::Semaphore(self.image_semaphore));
        self.device
            .push_dropped_object(VulkanObject::Semaphore(self.drawing_semaphore));
        self.device
            .push_dropped_object(VulkanObject::Fence(self.drawing_fence));
        self.device
            .push_dropped_object(VulkanObject::CommandPool(self.command_pool));
    }
}
