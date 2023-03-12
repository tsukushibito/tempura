use std::{
    hash::Hash,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use ash::vk;

use crate::{Device, VulkanObject};

pub struct RenderTarget {
    pub(crate) extent: vk::Extent2D,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) views: Vec<vk::ImageView>,
    pub(crate) attachments: Vec<vk::AttachmentDescription>,
    pub(crate) available_semaphore: vk::Semaphore,
    pub(crate) render_finished_semaphore: vk::Semaphore,

    id: usize,
    device: Rc<Device>,
    is_swapchain_image: bool,
}

impl RenderTarget {
    fn get_id() -> usize {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        COUNTER.fetch_add(1, Ordering::Relaxed)
    }

    // pub fn new(device: &Rc<Device>, extent: vk::Extent2D, format: vk::Format) -> Self {
    //     Self {
    //         extent,
    //         images: todo!(),
    //         views: todo!(),
    //         attachments: todo!(),
    //         available_semaphore: vk::Semaphore::null(),
    //         render_finished_semaphore: vk::Semaphore::null(),
    //         id: Self::get_id(),
    //         device: todo!(),
    //         is_swapchain_image: todo!(),
    //     }
    // }

    pub fn new_from_swapchain_image(
        device: &Rc<Device>,
        extent: vk::Extent2D,
        format: vk::Format,
        image: vk::Image,
    ) -> Self {
        unsafe {
            let info = vk::ImageViewCreateInfo::builder()
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::R,
                    g: vk::ComponentSwizzle::G,
                    b: vk::ComponentSwizzle::B,
                    a: vk::ComponentSwizzle::A,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .image(image)
                .build();
            let view = device
                .device
                .create_image_view(&info, None)
                .expect("Create image view error.");

            let attachment_desc = vk::AttachmentDescription::builder()
                .format(format)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .build();

            let semaphore_create_info = vk::SemaphoreCreateInfo::default();
            let available_semaphore = device
                .device
                .create_semaphore(&semaphore_create_info, None)
                .expect("create_semaphore failed.");
            let render_finished_semaphore = device
                .device
                .create_semaphore(&semaphore_create_info, None)
                .expect("create_semaphore failed.");

            Self {
                extent,
                images: vec![image],
                views: vec![view],
                attachments: vec![attachment_desc],
                available_semaphore,
                render_finished_semaphore,
                id: Self::get_id(),
                device: device.clone(),
                is_swapchain_image: true,
            }
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

impl Drop for RenderTarget {
    fn drop(&mut self) {
        self.views.iter().for_each(|&view| {
            self.device
                .push_dropped_object(VulkanObject::ImageView(view))
        });
        self.device
            .push_dropped_object(VulkanObject::Semaphore(self.available_semaphore));
        self.device
            .push_dropped_object(VulkanObject::Semaphore(self.render_finished_semaphore));
        if !self.is_swapchain_image {
            self.images
                .iter()
                .for_each(|&image| self.device.push_dropped_object(VulkanObject::Image(image)))
        }
    }
}

impl Hash for RenderTarget {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.images.hash(state);
        self.views.hash(state);
        self.attachments.iter().for_each(|&attachment| {
            attachment.final_layout.hash(state);
            attachment.flags.hash(state);
            attachment.format.hash(state);
            attachment.samples.hash(state);
            attachment.load_op.hash(state);
            attachment.store_op.hash(state);
            attachment.stencil_load_op.hash(state);
            attachment.stencil_store_op.hash(state);
            attachment.initial_layout.hash(state);
            attachment.final_layout.hash(state);
        });
    }
}
