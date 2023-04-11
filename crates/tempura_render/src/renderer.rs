use std::rc::Rc;

use ash::vk;

use crate::Device;
use crate::{
    common::{QueueFamilyIndices, Window},
    swapchain::Swapchain,
};

pub struct Renderer {
    device: Rc<Device>,
    swapchain: Swapchain,
    graphics_command_pool: vk::CommandPool,
    graphics_command_buffers: Vec<vk::CommandBuffer>,
    present_command_pool: Option<vk::CommandPool>,
    present_command_buffers: Option<Vec<vk::CommandBuffer>>,
}

impl Renderer {
    pub fn new<T>(device: &Rc<Device>, window: &T) -> Result<Self, Box<dyn std::error::Error>>
    where
        T: Window,
    {
        let surface = unsafe {
            ash_window::create_surface(
                &device.entry,
                &device.instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        let swapchain = Swapchain::new(&device, &surface, window)?;
        let (graphics_command_pool, present_command_pool) =
            create_command_pools(&device, &device.queue_family_indices)?;
        let graphics_command_buffers = allocate_command_buffers(
            &device,
            graphics_command_pool,
            swapchain.framebuffers.len() as u32,
        )?;

        let present_command_buffers = if let Some(command_pool) = present_command_pool {
            Some(allocate_command_buffers(
                &device,
                command_pool,
                swapchain.framebuffers.len() as u32,
            )?)
        } else {
            None
        };

        Ok(Self {
            device: device.clone(),
            swapchain,
            graphics_command_pool,
            graphics_command_buffers,
            present_command_pool,
            present_command_buffers,
        })
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .device_wait_idle()
                .expect("device_wait_idle error");

            let swapchain_loader =
                ash::extensions::khr::Swapchain::new(&self.device.instance, &self.device.device);
            swapchain_loader.destroy_swapchain(self.swapchain.swapchain, None);

            for &framebuffer in &self.swapchain.framebuffers {
                self.device.device.destroy_framebuffer(framebuffer, None)
            }

            self.device
                .device
                .destroy_render_pass(self.swapchain.render_pass, None);

            for &image_view in &self.swapchain.image_views {
                self.device.device.destroy_image_view(image_view, None);
            }

            let surface_loader =
                ash::extensions::khr::Surface::new(&self.device.entry, &self.device.instance);
            surface_loader.destroy_surface(self.swapchain.surface, None);

            self.device
                .device
                .destroy_command_pool(self.graphics_command_pool, None);
            if let Some(pool) = self.present_command_pool {
                self.device.device.destroy_command_pool(pool, None)
            };
        }
    }
}

fn create_command_pool(
    device: &Device,
    queue_family_index: u32,
) -> Result<vk::CommandPool, Box<dyn std::error::Error>> {
    let command_pool_create_info =
        vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family_index);

    let command_pool = unsafe {
        device
            .device
            .create_command_pool(&command_pool_create_info, None)?
    };
    Ok(command_pool)
}

fn create_command_pools(
    device: &Device,
    queue_family_indices: &QueueFamilyIndices,
) -> Result<(vk::CommandPool, Option<vk::CommandPool>), Box<dyn std::error::Error>> {
    let graphics_family = queue_family_indices.graphics_family.unwrap();
    let present_family = queue_family_indices.present_family.unwrap();

    let graphics_command_pool = create_command_pool(device, graphics_family)?;

    let present_command_pool = if graphics_family != present_family {
        Some(create_command_pool(device, present_family)?)
    } else {
        None
    };

    Ok((graphics_command_pool, present_command_pool))
}

fn allocate_command_buffers(
    device: &Device,
    command_pool: vk::CommandPool,
    buffer_count: u32,
) -> Result<Vec<vk::CommandBuffer>, Box<dyn std::error::Error>> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(buffer_count);

    let command_buffers = unsafe {
        device
            .device
            .allocate_command_buffers(&command_buffer_allocate_info)?
    };

    Ok(command_buffers)
}
