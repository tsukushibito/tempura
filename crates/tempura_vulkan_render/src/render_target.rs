use std::rc::Rc;

use ash::vk;

use crate::Device;

pub struct RenderTarget {
    pub(crate) extent: vk::Extent2D,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) views: Vec<vk::ImageView>,
    pub(crate) attachments: Vec<vk::AttachmentDescription>,

    device: Rc<Device>,
    is_swapchain_image: bool,
}

impl RenderTarget {
    pub fn new() -> Self {
        Self {
            extent: todo!(),
            images: todo!(),
            views: todo!(),
            attachments: todo!(),
            device: todo!(),
            is_swapchain_image: todo!(),
        }
    }
}

impl Drop for RenderTarget {
    fn drop(&mut self) {
        let images = self.images.to_owned();
        let views = self.views.to_owned();
        self.device.request_destroy(Box::new(move |context| unsafe {
            views
                .iter()
                .for_each(|&view| context.device.destroy_image_view(view, None));
            images
                .iter()
                .for_each(|&image| context.device.destroy_image(image, None));
        }));
    }
}
