use std::rc::Rc;

use ash::vk;

use crate::{Device, TvResult};

pub struct RenderPass {
    device: Rc<Device>,
    render_pass: vk::RenderPass,
}

impl RenderPass {
    pub(crate) fn new(
        device: &Rc<Device>,
        attachments: &[vk::AttachmentDescription],
        subpasses: &[vk::SubpassDescription],
        dependencies: &[vk::SubpassDependency],
    ) -> TvResult<Self> {
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies)
            .build();
        let render_pass = unsafe { device.handle().create_render_pass(&info, None) }?;
        Ok(Self {
            device: device.clone(),
            render_pass,
        })
    }

    pub(crate) fn handle(&self) -> vk::RenderPass {
        self.render_pass
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().device_wait_idle().unwrap();
        }
        unsafe {
            self.device
                .handle()
                .destroy_render_pass(self.render_pass, None);
        }
    }
}
