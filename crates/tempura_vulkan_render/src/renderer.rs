use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use ash::{prelude::VkResult, vk};

use crate::{RenderTarget, Swapchain, VulkanObject};

use super::Device;

pub struct Renderer {
    pub(crate) device: Rc<Device>,

    render_pass: Cell<vk::RenderPass>,
    framebuffers: RefCell<HashMap<usize, vk::Framebuffer>>,
}

impl Renderer {
    pub fn new(device: &Rc<Device>) -> Self {
        Renderer {
            device: device.clone(),
            render_pass: Cell::default(),
            framebuffers: RefCell::default(),
        }
    }

    pub fn render(&self, swapchain: &Swapchain) {
        unsafe {
            let result = swapchain.acquire_next_render_target();
            if result.is_none() {
                return;
            }
            let (render_target, image_index, frame_data) = result.unwrap();

            let mut render_pass = self.render_pass.get();
            if render_pass == vk::RenderPass::null() {
                render_pass = create_render_pass(&self.device.device, &render_target);
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
                .wait_for_fences(&[frame_data.drawing_fence], true, std::u64::MAX)
                .expect("Wait for fence failed.");
            device
                .reset_fences(&[frame_data.drawing_fence])
                .expect("Reset fences failed.");

            device
                .reset_command_pool(
                    frame_data.command_pool,
                    vk::CommandPoolResetFlags::RELEASE_RESOURCES,
                )
                .expect("reset command pool failed.");

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            device
                .begin_command_buffer(frame_data.command_buffers[0], &command_buffer_begin_info)
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
                frame_data.command_buffers[0],
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            device.cmd_end_render_pass(frame_data.command_buffers[0]);

            device
                .end_command_buffer(frame_data.command_buffers[0])
                .expect("End commandbuffer failed.");

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[frame_data.image_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[frame_data.command_buffers[0]])
                .signal_semaphores(&[frame_data.drawing_semaphore])
                .build();

            device
                .queue_submit(
                    self.device.graphics_queue,
                    &[submit_info],
                    frame_data.drawing_fence,
                )
                .expect("Queue submit failed.");

            swapchain.present(image_index, &frame_data)
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .push_dropped_object(VulkanObject::RenderPass(self.render_pass.get()));
            let framebuffers = self.framebuffers.borrow();
            framebuffers.iter().for_each(|(_, &framebuffer)| {
                self.device
                    .push_dropped_object(VulkanObject::Framebuffer(framebuffer))
            });
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
