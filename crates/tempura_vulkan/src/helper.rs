use ash::vk;

use crate::Swapchain;

pub fn attachments_for_swapchain(swapchain: &Swapchain) -> Vec<vk::AttachmentDescription> {
    vec![vk::AttachmentDescription::builder()
        .format(swapchain.image_format())
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
        .build()]
}
