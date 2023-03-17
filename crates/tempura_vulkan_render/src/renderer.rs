use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use ash::{prelude::VkResult, vk};

use crate::RenderTarget;

use super::Device;

pub struct Renderer {
    pub(crate) device: Rc<Device>,

    command_pool: vk::CommandPool,
    _setup_command_buffer: vk::CommandBuffer,
    draw_command_buffer: vk::CommandBuffer,
    render_fence: vk::Fence,

    render_pass: Cell<vk::RenderPass>,
    framebuffers: RefCell<HashMap<usize, vk::Framebuffer>>,
}

impl Renderer {
    pub fn new(device: &Rc<Device>) -> Self {
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

        Renderer {
            device: device.clone(),
            command_pool,
            _setup_command_buffer: setup_command_buffer,
            draw_command_buffer,
            render_fence,
            render_pass: Cell::default(),
            framebuffers: RefCell::default(),
        }
    }

    pub fn render(&self, render_target: &RenderTarget) {
        unsafe {
            self.device.destroy_dropped_objects();

            let mut render_pass = self.render_pass.get();
            if render_pass == vk::RenderPass::null() {
                render_pass = create_render_pass(&self.device.device, render_target);
                self.render_pass.set(render_pass);
            }

            let mut framebuffers = self.framebuffers.borrow_mut();
            let framebuffer = framebuffers.entry(render_target.id()).or_insert_with(|| {
                let create_info = vk::FramebufferCreateInfo::builder()
                    .width(render_target.extent.width)
                    .height(render_target.extent.height)
                    .layers(1)
                    .render_pass(render_pass)
                    .attachments(&render_target.views)
                    .build();
                self.device
                    .device
                    .create_framebuffer(&create_info, None)
                    .expect("crate framebuffer error.")
            });

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

            let render_area = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: render_target.extent,
            };
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .clear_values(&clear_values)
                .render_pass(render_pass)
                .render_area(render_area)
                .framebuffer(*framebuffer)
                .build();

            device.cmd_begin_render_pass(
                self.draw_command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            device.cmd_end_render_pass(self.draw_command_buffer);

            device
                .end_command_buffer(self.draw_command_buffer)
                .expect("End commandbuffer failed.");

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[render_target.available_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.draw_command_buffer])
                .signal_semaphores(&[render_target.render_finished_semaphore])
                .build();

            device
                .queue_submit(self.device.render_queue, &[submit_info], self.render_fence)
                .expect("Queue submit failed.");
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let device = &self.device.device;
            device.device_wait_idle().unwrap();
            device.destroy_render_pass(self.render_pass.get(), None);
            let framebuffers = self.framebuffers.borrow();
            framebuffers.iter().for_each(|(_, framebuffer)| {
                device.destroy_framebuffer(*framebuffer, None);
            });
            device.destroy_fence(self.render_fence, None);
            device.destroy_command_pool(self.command_pool, None);
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

fn create_render_pass(device: &ash::Device, render_target: &RenderTarget) -> vk::RenderPass {
    unsafe {
        let color_attachment_ref = vk::AttachmentReference::builder()
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();
        let subpass = vk::SubpassDescription::builder()
            .color_attachments(&[color_attachment_ref])
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build();
        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&render_target.attachments)
            .subpasses(&[subpass])
            .build();

        device
            .create_render_pass(&create_info, None)
            .expect("create_render_pass error")
    }
}
