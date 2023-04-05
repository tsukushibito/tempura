use ash::vk;
use tempura_render as tr;

pub struct RenderTarget {
    pub(crate) extent: vk::Extent2D,
    pub(crate) images: Vec<vk::Image>,
    pub(crate) views: Vec<vk::ImageView>,
    pub(crate) attachments: Vec<vk::AttachmentDescription>,
}

impl tr::RenderTarget for RenderTarget {}
