use std::rc::Rc;

use ash::vk;

use crate::{Device, Image, TvResult};

pub struct ImageView {
    device: Rc<Device>,
    image: Rc<Image>,
    image_view: vk::ImageView,
}

impl ImageView {
    pub(crate) fn new(
        device: &Rc<Device>,
        image: &Rc<Image>,
        view_type: vk::ImageViewType,
        format: vk::Format,
        components: vk::ComponentMapping,
        subresource_range: vk::ImageSubresourceRange,
    ) -> TvResult<Self> {
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image.handle())
            .view_type(view_type)
            .format(format)
            .components(components)
            .subresource_range(subresource_range)
            .build();

        let image_view = unsafe {
            device
                .handle()
                .create_image_view(&image_view_create_info, None)?
        };

        Ok(Self {
            device: device.clone(),
            image: image.clone(),
            image_view,
        })
    }

    pub fn handle(&self) -> vk::ImageView {
        self.image_view
    }

    pub fn image(&self) -> &Rc<Image> {
        &self.image
    }
}

impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.device.handle().device_wait_idle().unwrap();
        }
        unsafe {
            self.device
                .handle()
                .destroy_image_view(self.image_view, None);
        }
    }
}
