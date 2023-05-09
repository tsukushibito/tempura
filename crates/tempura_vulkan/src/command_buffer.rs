use std::rc::Rc;

use ash::vk;

use crate::CommandPool;
use crate::Device;
use crate::Framebuffer;
use crate::RenderPass;
use crate::TvResult;

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

    pub fn handle(&self) -> vk::CommandBuffer {
        self.command_buffer
    }

    pub fn begin(
        &self,
        flags: vk::CommandBufferUsageFlags,
        inheritance_info: Option<&vk::CommandBufferInheritanceInfo>,
    ) -> TvResult<()> {
        let mut command_buffer_begin_info = vk::CommandBufferBeginInfo::builder().flags(flags);
        if let Some(inheritance_info) = inheritance_info {
            command_buffer_begin_info =
                command_buffer_begin_info.inheritance_info(inheritance_info);
        }

        unsafe {
            self.device
                .handle()
                .begin_command_buffer(self.command_buffer, &command_buffer_begin_info)?
        };

        Ok(())
    }

    pub fn end(&self) -> TvResult<()> {
        unsafe {
            self.device
                .handle()
                .end_command_buffer(self.command_buffer)?
        };

        Ok(())
    }

    pub fn begin_render_pass(
        &self,
        render_pass: &RenderPass,
        framebuffer: &Framebuffer,
        render_area: &vk::Rect2D,
        clear_values: &[vk::ClearValue],
        contents: vk::SubpassContents,
    ) {
        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass.handle())
            .framebuffer(framebuffer.handle())
            .render_area(*render_area)
            .clear_values(clear_values)
            .build();

        unsafe {
            self.device.handle().cmd_begin_render_pass(
                self.command_buffer,
                &render_pass_begin_info,
                contents,
            )
        };
    }

    pub fn end_render_pass(&self) {
        unsafe {
            self.device
                .handle()
                .cmd_end_render_pass(self.command_buffer);
        }
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
