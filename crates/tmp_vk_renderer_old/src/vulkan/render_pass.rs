use std::rc::Rc;

use ash::vk;

use super::device::Device;
use crate::TmpResult;

pub struct RenderPass {
    device: Rc<Device>,
    render_pass: vk::RenderPass,
    attachments: Vec<vk::AttachmentDescription>,
    subpasses: Vec<vk::SubpassDescription>,
    dependencies: Vec<vk::SubpassDependency>,
}

impl RenderPass {
    pub fn new(
        device: &Rc<Device>,
        attachments: &[vk::AttachmentDescription],
        subpasses: &[vk::SubpassDescription],
        dependencies: &[vk::SubpassDependency],
    ) -> TmpResult<Self> {
        let info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments)
            .subpasses(subpasses)
            .dependencies(dependencies)
            .build();
        let render_pass = unsafe { device.handle().create_render_pass(&info, None) }?;
        Ok(Self {
            device: device.clone(),
            render_pass,
            attachments: attachments.to_vec(),
            subpasses: subpasses.to_vec(),
            dependencies: dependencies.to_vec(),
        })
    }

    pub fn handle(&self) -> vk::RenderPass {
        self.render_pass
    }

    pub fn attachments(&self) -> &[vk::AttachmentDescription] {
        &self.attachments
    }

    pub fn subpasses(&self) -> &[vk::SubpassDescription] {
        &self.subpasses
    }

    pub fn dependencies(&self) -> &[vk::SubpassDependency] {
        &self.dependencies
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
