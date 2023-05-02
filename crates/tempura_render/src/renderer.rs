use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use ash::vk;
use tempura_vulkan::{
    CommandBuffer, CommandPool, Device, Fence, QueueFamily, Semaphore, Swapchain, Window,
};

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct FrameData {
    image_available_semaphore: Semaphore,
    render_finished_semaphore: Semaphore,
    in_flight_fence: Fence,
    command_pool: Rc<CommandPool>,
    command_buffer: CommandBuffer,
}

pub struct Renderer<T: Window> {
    device: Rc<Device>,
    swapchain: RefCell<Swapchain>,
    window: Rc<T>,
    frame_datas: [FrameData; MAX_FRAMES_IN_FLIGHT],
    current_frame: Cell<usize>,
}

impl<T: Window> Renderer<T> {
    pub fn new(device: &Rc<Device>, window: &Rc<T>) -> Result<Self, Box<dyn std::error::Error>> {
        let swapchain = device.create_swapchain(window)?;

        let frame_datas = core::array::from_fn(|_| {
            let image_available_semaphore = device.create_semaphore().unwrap();
            let render_finished_semaphore = device.create_semaphore().unwrap();
            let in_flight_fence = device.create_fence(true).unwrap();
            let command_pool = Rc::new(device.create_command_pool(QueueFamily::Graphics).unwrap());
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
            device: device.clone(),
            swapchain: RefCell::new(swapchain),
            window: window.clone(),
            frame_datas,
            current_frame: Cell::new(0),
        })
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        let frame_data = &self.frame_datas[self.current_frame.get()];
        frame_data.in_flight_fence.wait()?;
        let result = self
            .swapchain
            .borrow()
            .acquire_next_image(&frame_data.image_available_semaphore);

        let index = match result {
            Ok((image_index, _)) => image_index,
            Err(e) => {
                let vk_result = e.downcast_ref::<vk::Result>();
                match vk_result {
                    Some(_) => {
                        self.swapchain.replace(
                            self.device
                                .create_swapchain(&self.window)
                                .expect("Failed to create swapchain"),
                        );
                        return Ok(());
                    }
                    None => return Err(e),
                }
            }
        };

        let command_buffer = &frame_data.command_buffer;
        Ok(())
    }
}
