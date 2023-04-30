use std::{cell::Cell, rc::Rc};

use ash::vk;
use tempura_vulkan::{
    CommandBuffer, CommandPool, Fence, QueueFamily, Semaphore, Swapchain, VulkanDevice, Window,
};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct FrameData {
    image_available_semaphore: Rc<Semaphore>,
    render_finished_semaphore: Rc<Semaphore>,
    in_flight_fence: Rc<Fence>,
    command_pool: Rc<CommandPool>,
    command_buffer: Rc<CommandBuffer>,
}

pub struct Renderer<T: Window> {
    vulkan_device: Rc<VulkanDevice>,
    swapchain: Rc<Swapchain<T>>,
    frame_datas: [FrameData; MAX_FRAMES_IN_FLIGHT],
    current_frame: Cell<usize>,
}

impl<T: Window> Renderer<T> {
    pub fn new(
        render_device: &Rc<VulkanDevice>,
        window: &Rc<Box<T>>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let swapchain = render_device.create_swapchain(window)?;

        let frame_datas = core::array::from_fn(|_| {
            let image_available_semaphore = render_device.create_semaphore().unwrap();
            let render_finished_semaphore = render_device.create_semaphore().unwrap();
            let in_flight_fence = render_device.create_fence(true).unwrap();
            let command_pool = render_device
                .create_command_pool(QueueFamily::Graphics)
                .unwrap();
            let command_buffer = command_pool
                .allocate_command_buffers(vk::CommandBufferLevel::PRIMARY, 1)
                .unwrap()
                .remove(0);

            FrameData {
                image_available_semaphore,
                render_finished_semaphore,
                in_flight_fence,
                command_pool,
                command_buffer,
            }
        });

        Ok(Self {
            vulkan_device: render_device.clone(),
            swapchain,
            frame_datas,
            current_frame: Cell::new(0),
        })
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        let frame_data = &self.frame_datas[self.current_frame.get()];
        frame_data.in_flight_fence.wait()?;
        self.swapchain
            .acquire_next_image(&frame_data.image_available_semaphore)?;

        let command_buffer = &frame_data.command_buffer;
        Ok(())
    }
}
