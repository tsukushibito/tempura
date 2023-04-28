use std::rc::Rc;

use ash::vk;

use tempura_graphics::{CommandPool, GraphicsDevice, QueueFamily, Swapchain, Window};

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
    render_device: Rc<GraphicsDevice>,
    swapchain: Swapchain,
    frame_datas: Vec<FrameData>,
}

impl Renderer {
    pub fn new<T>(
        render_device: &Rc<GraphicsDevice>,
        window: &T,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: Window,
    {
        let swapchain = render_device.create_swapchain(window)?;
        let graphics_command_pool = render_device
            .create_command_pool(QueueFamily::Graphics)?
            .unwrap();
        let present_command_pool = render_device.create_command_pool(QueueFamily::Present)?;
        let graphics_command_buffers = graphics_command_pool.allocate_command_buffers(
            vk::CommandBufferLevel::PRIMARY,
            swapchain.framebuffer_count() as u32,
        )?;

        let present_command_buffers = if let Some(present_command_pool) = present_command_pool {
            Some(present_command_pool.allocate_command_buffers(
                vk::CommandBufferLevel::PRIMARY,
                swapchain.framebuffer_count() as u32,
            )?)
        } else {
            None
        };

        Ok(Self {
            render_device: render_device.clone(),
            swapchain,
            graphics_command_pool: Rc::new(graphics_command_pool),
            graphics_command_buffers,
            present_command_pool: Rc::new(present_command_pool.unwrap()),
            present_command_buffers,
        })
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.render_device
                .device
                .destroy_command_pool(self.graphics_command_pool, None);
            if let Some(pool) = self.present_command_pool {
                self.render_device.device.destroy_command_pool(pool, None)
            };
        }
    }
}

fn create_command_pool(
    render_device: &RenderDevice,
    queue_family_index: u32,
) -> Result<vk::CommandPool, Box<dyn std::error::Error>> {
    let command_pool_create_info =
        vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family_index);

    let command_pool = unsafe {
        render_device
            .device
            .create_command_pool(&command_pool_create_info, None)?
    };
    Ok(command_pool)
}

fn create_command_pools(
    render_device: &RenderDevice,
    queue_family_indices: &QueueFamilyIndices,
) -> Result<(vk::CommandPool, Option<vk::CommandPool>), Box<dyn std::error::Error>> {
    let graphics_family = queue_family_indices.graphics_family.unwrap();
    let present_family = queue_family_indices.present_family.unwrap();

    let graphics_command_pool = create_command_pool(render_device, graphics_family)?;

    let present_command_pool = if graphics_family != present_family {
        Some(create_command_pool(render_device, present_family)?)
    } else {
        None
    };

    Ok((graphics_command_pool, present_command_pool))
}

fn allocate_command_buffers(
    render_device: &RenderDevice,
    command_pool: vk::CommandPool,
    buffer_count: u32,
) -> Result<Vec<vk::CommandBuffer>, Box<dyn std::error::Error>> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(buffer_count);

    let command_buffers = unsafe {
        render_device
            .device
            .allocate_command_buffers(&command_buffer_allocate_info)?
    };

    Ok(command_buffers)
}
