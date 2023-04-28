use std::rc::Rc;

use ash::{extensions, vk};

use crate::command_buffer::CommandBuffer;
use crate::command_pool::CommandPool;
use crate::common::Window;
use crate::graphics_device::GraphicsDevice;

struct FrameData {
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
    image_index: u32,
    graphics_command_pool: Rc<CommandPool>,
    graphics_command_buffer: Rc<CommandBuffer>,
    present_command_pool: Rc<CommandPool>,
    present_command_buffer: Rc<CommandBuffer>,
}

pub struct Swapchain {
    graphics_device: Rc<GraphicsDevice>,
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    extent: vk::Extent2D,
    format: vk::Format,
}

impl Swapchain {
    pub fn new<T>(
        graphics_device: &Rc<GraphicsDevice>,
        surface: &vk::SurfaceKHR,
        window: &T,
    ) -> Result<Swapchain, Box<dyn std::error::Error>>
    where
        T: Window,
    {
        let surface_loader = graphics_device.surface_loader();
        let physical_device = graphics_device.physical_device();
        let surface_format = choose_swapchain_format(&surface_loader, &physical_device, surface)?;

        let present_mode =
            choose_swapchain_present_mode(&surface_loader, &physical_device, surface)?;

        let surface_capabilities = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, *surface)?
        };
        let image_count = std::cmp::min(
            surface_capabilities.min_image_count + 1,
            surface_capabilities.max_image_count,
        );
        let surface_resolution = if surface_capabilities.current_extent.width == std::u32::MAX {
            let (width, height) = window.window_size();
            vk::Extent2D { width, height }
        } else {
            surface_capabilities.current_extent
        };

        let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(surface_capabilities.current_extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let queue_family_indices = graphics_device.queue_family_indices();
        let queue_family_indices = [
            queue_family_indices.graphics_family,
            queue_family_indices.present_family,
        ];

        if queue_family_indices[0] != queue_family_indices[1] {
            swapchain_create_info = swapchain_create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            swapchain_create_info =
                swapchain_create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }

        let swapchain_create_info = swapchain_create_info.build();

        let device = graphics_device.device();
        let swapchain_loader = graphics_device.swapchain_loader();
        let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
        let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let image_views = images
            .iter()
            .map(|&image| {
                let info = vk::ImageViewCreateInfo::builder()
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(surface_format.format)
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
                unsafe { device.create_image_view(&info, None).unwrap() }
            })
            .collect::<Vec<vk::ImageView>>();

        let color_attachment_desc = vk::AttachmentDescription::builder()
            .format(surface_format.format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_ref = vk::AttachmentReference::builder()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build();

        let subpass_desc = vk::SubpassDescription::builder()
            .color_attachments(&[color_attachment_ref])
            .build();

        let render_pass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment_desc])
            .subpasses(&[subpass_desc])
            .build();

        let render_pass = unsafe { device.create_render_pass(&render_pass_create_info, None)? };

        let framebuffers = image_views
            .iter()
            .map(|&view| {
                let framebuffer_create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&[view])
                    .width(surface_resolution.width)
                    .height(surface_resolution.height)
                    .layers(1)
                    .build();
                unsafe {
                    device
                        .create_framebuffer(&framebuffer_create_info, None)
                        .unwrap()
                }
            })
            .collect::<Vec<vk::Framebuffer>>();

        Ok(Self {
            graphics_device: graphics_device.clone(),
            surface: *surface,
            swapchain,
            images,
            image_views,
            render_pass,
            framebuffers,
            extent: swapchain_create_info.image_extent,
            format: swapchain_create_info.image_format,
        })
    }

    pub fn framebuffer_count(&self) -> usize {
        self.framebuffers.len()
    }

    pub fn acquire_next_image(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let (index, _) = unsafe {
            self.graphics_device.swapchain_loader().acquire_next_image(
                self.swapchain,
                1000 * 1000,
                vk::Semaphore::null(),
                vk::Fence::null(),
            )?
        };

        Ok(index)
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        let device = self.graphics_device.device();
        unsafe { device.device_wait_idle().expect("device_wait_idle error") };

        let swapchain_loader = self.graphics_device.swapchain_loader();
        unsafe { swapchain_loader.destroy_swapchain(self.swapchain, None) };

        for &framebuffer in &self.framebuffers {
            unsafe { device.destroy_framebuffer(framebuffer, None) }
        }

        unsafe { device.destroy_render_pass(self.render_pass, None) };

        for &image_view in &self.image_views {
            unsafe { device.destroy_image_view(image_view, None) };
        }

        let surface_loader = self.graphics_device.surface_loader();
        unsafe { surface_loader.destroy_surface(self.surface, None) };
    }
}

fn choose_swapchain_format(
    surface_loader: &extensions::khr::Surface,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> Result<vk::SurfaceFormatKHR, Box<dyn std::error::Error>> {
    let formats =
        unsafe { surface_loader.get_physical_device_surface_formats(*physical_device, *surface)? };

    for &format in &formats {
        if format.format == vk::Format::B8G8R8A8_SRGB
            && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return Ok(format);
        }
    }

    Ok(formats[0])
}

fn choose_swapchain_present_mode(
    surface_loader: &extensions::khr::Surface,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> Result<vk::PresentModeKHR, Box<dyn std::error::Error>> {
    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(*physical_device, *surface)?
    };

    for mode in present_modes {
        if mode == vk::PresentModeKHR::MAILBOX {
            return Ok(mode);
        }
    }

    Ok(vk::PresentModeKHR::FIFO)
}
