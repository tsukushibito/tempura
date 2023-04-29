use std::rc::Rc;

use ash::vk;

use tempura_vulkan::{RcWindow, Swapchain, VulkanDevice, Window};

pub struct Renderer {
    vulkan_device: Rc<VulkanDevice>,
    swapchain: Rc<Swapchain>,
}

impl Renderer {
    pub fn new(
        render_device: &Rc<VulkanDevice>,
        window: &RcWindow,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let swapchain = render_device.create_swapchain(window)?;

        Ok(Self {
            vulkan_device: render_device.clone(),
            swapchain,
        })
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {}
}
