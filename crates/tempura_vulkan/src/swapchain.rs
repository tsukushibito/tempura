use std::rc::Rc;

use ash::{extensions, vk};

use crate::command_buffer::CommandBuffer;
use crate::command_pool::CommandPool;
use crate::common::Window;
use crate::vulkan_device::VulkanDevice;

pub struct FrameData {
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    in_flight_fence: vk::Fence,
    image: vk::Image,
    image_view: vk::ImageView,
    graphics_command_pool: Rc<CommandPool>,
    graphics_command_buffer: Rc<CommandBuffer>,
    present_command_pool: Rc<CommandPool>,
    present_command_buffer: Rc<CommandBuffer>,
}

pub struct Swapchain {
    vulkan_device: Rc<VulkanDevice>,
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    image_format: vk::Format,
    image_color_space: vk::ColorSpaceKHR,
    image_extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    frame_datas: Vec<FrameData>,
}

impl Swapchain {
    pub fn new<T>(
        vulkan_device: &Rc<VulkanDevice>,
        surface: &vk::SurfaceKHR,
        window: &T,
    ) -> Result<Swapchain, Box<dyn std::error::Error>>
    where
        T: Window,
    {
        let surface_loader = vulkan_device.surface_loader();
        let physical_device = vulkan_device.physical_device();
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
            .image_extent(surface_resolution)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        let queue_family_indices = vulkan_device.queue_family_indices();
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

        let device = vulkan_device.device();
        let swapchain_loader = vulkan_device.swapchain_loader();
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

        let frame_datas = images
            .iter()
            .zip(image_views.iter())
            .map(|(&image, &image_view)| {
                let graphics_command_pool =
                    Rc::new(CommandPool::new(vulkan_device, queue_family_indices[0]).unwrap());
                let graphics_command_buffers = graphics_command_pool
                    .allocate_command_buffers(vk::CommandBufferLevel::PRIMARY, 1)
                    .unwrap();
                let present_command_pool =
                    Rc::new(CommandPool::new(vulkan_device, queue_family_indices[1]).unwrap());
                let present_command_buffers = present_command_pool
                    .allocate_command_buffers(vk::CommandBufferLevel::PRIMARY, 1)
                    .unwrap();

                FrameData {
                    image_available_semaphore: vk::Semaphore::null(),
                    render_finished_semaphore: vk::Semaphore::null(),
                    in_flight_fence: vk::Fence::null(),
                    image,
                    image_view,
                    graphics_command_pool,
                    graphics_command_buffer: graphics_command_buffers[0].clone(),
                    present_command_pool,
                    present_command_buffer: present_command_buffers[0].clone(),
                }
            })
            .collect::<Vec<FrameData>>();

        Ok(Self {
            vulkan_device: vulkan_device.clone(),
            surface: *surface,
            swapchain,
            image_extent: surface_resolution,
            image_format: surface_format.format,
            image_color_space: surface_format.color_space,
            present_mode,
            frame_datas,
        })
    }

    pub fn image_count(&self) -> usize {
        self.frame_datas.len()
    }

    pub fn image_extent(&self) -> vk::Extent2D {
        self.image_extent
    }

    pub fn image_format(&self) -> vk::Format {
        self.image_format
    }

    pub fn image_color_space(&self) -> vk::ColorSpaceKHR {
        self.image_color_space
    }

    pub fn present_mode(&self) -> vk::PresentModeKHR {
        self.present_mode
    }

    pub fn acquire_next_image(&self) -> Result<u32, Box<dyn std::error::Error>> {
        let (index, _) = unsafe {
            self.vulkan_device.swapchain_loader().acquire_next_image(
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
        let device = self.vulkan_device.device();
        unsafe { device.device_wait_idle().expect("device_wait_idle error") };

        let swapchain_loader = self.vulkan_device.swapchain_loader();
        unsafe { swapchain_loader.destroy_swapchain(self.swapchain, None) };

        for frame_data in &self.frame_datas {
            unsafe { device.destroy_fence(frame_data.in_flight_fence, None) };
            unsafe { device.destroy_semaphore(frame_data.render_finished_semaphore, None) };
            unsafe { device.destroy_semaphore(frame_data.image_available_semaphore, None) };
            unsafe { device.destroy_image_view(frame_data.image_view, None) };
        }

        let surface_loader = self.vulkan_device.surface_loader();
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
