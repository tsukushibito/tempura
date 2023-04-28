use std::rc::Rc;

use ash::vk;

use tempura_vulkan::{CommandPool, QueueFamily, Swapchain, VulkanDevice, Window};

struct FrameData {
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
    image_index: u32,
    graphics_command_pool: CommandPool,
    graphics_command_buffer: vk::CommandBuffer,
    present_command_pool: Option<CommandPool>,
    present_command_buffer: Option<vk::CommandBuffer>,
}

pub struct Renderer {
    vulkan_device: Rc<VulkanDevice>,
    swapchain: Rc<Swapchain>,
    frame_datas: Vec<FrameData>,
}

impl Renderer {
    pub fn new<T>(
        render_device: &Rc<VulkanDevice>,
        window: &T,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: Window,
    {
        let swapchain = render_device.create_swapchain(window)?;
        let graphics_command_pool =
            Rc::new(render_device.create_command_pool(QueueFamily::Graphics)?);
        let present_command_pool = render_device.create_command_pool(QueueFamily::Present)?;

        Ok(Self {
            vulkan_device: render_device.clone(),
            swapchain,
            frame_datas: vec![],
        })
    }

    pub fn render(&self) {}
}

impl Drop for Renderer {
    fn drop(&mut self) {}
}
