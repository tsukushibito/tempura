use std::rc::Rc;

use ash::{prelude::VkResult, vk};
use raw_window_handle::RawDisplayHandle;

use crate::RenderTarget;

use super::Device;

pub struct Renderer {
    pub(crate) device: Rc<Device>,

    present_queue: vk::Queue,
    present_semaphore: vk::Semaphore,
    render_semaphore: vk::Semaphore,
    command_pool: vk::CommandPool,
    _setup_command_buffer: vk::CommandBuffer,
    draw_command_buffer: vk::CommandBuffer,
    render_fence: vk::Fence,
}

impl Renderer {
    pub fn new(device: &Rc<Device>) -> Self {
        let present_queue = unsafe {
            device
                .device
                .get_device_queue(device.graphics_queue_family_index, 0)
        };
        let command_pool = create_command_pool(&device.device, device.graphics_queue_family_index)
            .expect("Create command pool error");
        let command_buffers = create_command_buffers(&device.device, &command_pool)
            .expect("Create command buffers error");
        let setup_command_buffer = command_buffers[0];
        let draw_command_buffer = command_buffers[1];
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();
        let render_fence = unsafe {
            device
                .device
                .create_fence(&fence_create_info, None)
                .expect("Create fence error")
        };
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let present_semaphore = unsafe {
            device
                .device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Create semaphore error")
        };
        let render_semaphore = unsafe {
            device
                .device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Create semaphore error")
        };

        Renderer {
            device: device.clone(),
            present_queue,
            present_semaphore,
            render_semaphore,
            command_pool,
            _setup_command_buffer: setup_command_buffer,
            draw_command_buffer,
            render_fence,
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device.device;
            device.device_wait_idle().unwrap();
            device.destroy_semaphore(self.present_semaphore, None);
            device.destroy_semaphore(self.render_semaphore, None);
            device.destroy_fence(self.render_fence, None);
            device.destroy_command_pool(self.command_pool, None);
        }
    }
}

impl tempura_render::Renderer for Renderer {
    type RenderTarget = RenderTarget;

    fn render(&self, render_target: &Self::RenderTarget) {
        unsafe {
            self.device.execute_destroy_request();

            let device = &self.device.device;
            device
                .wait_for_fences(&[self.render_fence], true, std::u64::MAX)
                .expect("Wait for fence failed.");
            device
                .reset_fences(&[self.render_fence])
                .expect("Reset fences failed.");

            device
                .reset_command_buffer(
                    self.draw_command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Reset command buffer failed.");

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            device
                .begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)
                .expect("Begin commandbuffer failed.");

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.1, 1.0],
                },
            }];

            device
                .end_command_buffer(self.draw_command_buffer)
                .expect("End commandbuffer failed.");

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[self.present_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.draw_command_buffer])
                .signal_semaphores(&[self.render_semaphore])
                .build();

            device
                .queue_submit(self.present_queue, &[submit_info], self.render_fence)
                .expect("Queue submit failed.");
        }
    }
}

fn create_command_pool(device: &ash::Device, queue_family_index: u32) -> VkResult<vk::CommandPool> {
    unsafe {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index)
            .build();
        device.create_command_pool(&create_info, None)
    }
}

fn create_command_buffers(
    device: &ash::Device,
    command_pool: &vk::CommandPool,
) -> VkResult<Vec<vk::CommandBuffer>> {
    unsafe {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(2)
            .command_pool(*command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();
        device.allocate_command_buffers(&allocate_info)
    }
}
