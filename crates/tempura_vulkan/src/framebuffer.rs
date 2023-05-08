use std::rc::Rc;

use ash::vk;

use crate::{Device, ImageView, RenderPass, TvResult};

pub struct Framebuffer {
    device: Rc<Device>,
    render_pass: Rc<RenderPass>,
    framebuffer: vk::Framebuffer,
    image_view: Rc<ImageView>,
    layers: u32,
}

impl Framebuffer {
    pub fn new(
        device: &Rc<Device>,
        render_pass: &Rc<RenderPass>,
        image_view: &Rc<ImageView>,
        layers: u32,
    ) -> TvResult<Self> {
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
            render_pass: render_pass.clone(),
            framebuffer,
            image_view: image_view.clone(),
            layers,
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
