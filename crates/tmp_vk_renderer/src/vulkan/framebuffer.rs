use std::rc::Rc;

use ash::vk;

use super::{
    device::Device, image_view::ImageView, render_pass::RenderPass,
};
use crate::TmpResult;

pub struct Framebuffer {
    device: Rc<Device>,
    _render_pass: Rc<RenderPass>,
    framebuffer: vk::Framebuffer,
    _image_view: Rc<ImageView>,
    _layers: u32,
}

impl Framebuffer {
    pub fn new(
        device: &Rc<Device>,
        render_pass: &Rc<RenderPass>,
        image_view: &Rc<ImageView>,
        layers: u32,
    ) -> TmpResult<Self> {
        let info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass.handle())
            .attachments(&[image_view.handle()])
            .width(image_view.image().extent().width)
            .height(image_view.image().extent().height)
            .layers(layers)
            .build();

        let framebuffer = unsafe { device.handle().create_framebuffer(&info, None)? };
        Ok(Self {
            device: device.clone(),
            _render_pass: render_pass.clone(),
            framebuffer,
            _image_view: image_view.clone(),
            _layers: layers,
        })
    }

    pub fn handle(&self) -> vk::Framebuffer {
        self.framebuffer
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().device_wait_idle().unwrap();
        }
        unsafe {
            self.device
                .handle()
                .destroy_framebuffer(self.framebuffer, None);
        }
    }
}
