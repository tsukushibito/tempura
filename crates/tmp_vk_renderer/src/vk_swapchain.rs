use std::rc::Rc;

use ash::{vk, Device};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{common::*, VkRenderer};

pub struct VkSwapchain {
    renderer: Rc<VkRenderer>,
    surface_loader: ash::extensions::khr::Surface,
    swapchain_loader: ash::extensions::khr::Swapchain,
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    image_format: vk::Format,
    image_color_space: vk::ColorSpaceKHR,
    pub(crate) image_extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    command_pools: Vec<vk::CommandPool>,
    pub(crate) command_buffers: Vec<vk::CommandBuffer>,
    // image_available_semaphores: Vec<vk::Semaphore>, // イメージ取得用セマフォ
    // render_finished_semaphores: Vec<vk::Semaphore>, // レンダリング完了用セマフォ
    // in_flight_fences: Vec<vk::Fence>,               // レンダリング操作の完了を追跡するフェンス
    // current_frame: usize,                           // 現在のフレームインデックス
}

impl VkSwapchain {
    pub fn new(
        renderer: &Rc<VkRenderer>,
        display_handle: &RawDisplayHandle,
        window_handle: &RawWindowHandle,
        window_width: u32,
        window_height: u32,
    ) -> TmpResult<Self> {
        let entry = &renderer.entry;
        let instance = &renderer.instance;
        let device = &renderer.device;
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        let (
            surface,
            swapchain,
            image_format,
            image_color_space,
            image_extent,
            present_mode,
            images,
            image_views,
            command_pools,
            command_buffers,
        ) = create_swapchain_resources(
            &renderer,
            &surface_loader,
            &swapchain_loader,
            display_handle,
            window_handle,
            window_width,
            window_height,
        )?;
        Ok(Self {
            renderer: renderer.clone(),
            surface_loader,
            swapchain_loader,
            surface,
            swapchain,
            image_format,
            image_color_space,
            image_extent,
            present_mode,
            images,
            image_views,
            command_pools,
            command_buffers,
        })
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        let entry = &self.renderer.entry;
        let instance = &self.renderer.instance;
        let device = &self.renderer.device;

        unsafe { device.device_wait_idle().expect("device_wait_idle error") };

        unsafe {
            self.swapchain_loader
                .destroy_swapchain(self.swapchain, None)
        };

        unsafe { self.surface_loader.destroy_surface(self.surface, None) };
    }
}

fn create_swapchain_resources(
    renderer: &VkRenderer,
    surface_loader: &ash::extensions::khr::Surface,
    swapchain_loader: &ash::extensions::khr::Swapchain,
    display_handle: &RawDisplayHandle,
    window_handle: &RawWindowHandle,
    window_width: u32,
    window_height: u32,
) -> TmpResult<(
    vk::SurfaceKHR,
    vk::SwapchainKHR,
    vk::Format,
    vk::ColorSpaceKHR,
    vk::Extent2D,
    vk::PresentModeKHR,
    Vec<vk::Image>,
    Vec<vk::ImageView>,
    Vec<vk::CommandPool>,
    Vec<vk::CommandBuffer>,
)> {
    let entry = &renderer.entry;
    let instance = &renderer.instance;
    let surface = unsafe {
        ash_window::create_surface(entry, instance, *display_handle, *window_handle, None)?
    };
    let physical_device = renderer.physical_device;

    let surface_format = choose_swapchain_format(&surface_loader, &physical_device, &surface)?;
    let present_mode = choose_swapchain_present_mode(&surface_loader, &physical_device, &surface)?;
    let surface_capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?
    };
    let image_count = std::cmp::min(
        surface_capabilities.min_image_count + 1,
        surface_capabilities.max_image_count,
    );
    let surface_resolution = if surface_capabilities.current_extent.width == std::u32::MAX {
        vk::Extent2D {
            width: window_width,
            height: window_height,
        }
    } else {
        surface_capabilities.current_extent
    };

    let mut swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
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

    let queue_family_indices = &renderer.queue_family_indices;
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

    let device = &renderer.device;
    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
    let image_views = images
        .iter()
        .map(|&image| {
            let view_type = vk::ImageViewType::TYPE_2D;
            let format = surface_format.format;
            let components = vk::ComponentMapping {
                r: vk::ComponentSwizzle::R,
                g: vk::ComponentSwizzle::G,
                b: vk::ComponentSwizzle::B,
                a: vk::ComponentSwizzle::A,
            };
            let subresource_range = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            };

            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(view_type)
                .format(format)
                .components(components)
                .subresource_range(subresource_range)
                .build();

            let image_view = unsafe { device.create_image_view(&image_view_create_info, None) };
            image_view.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
        })
        .collect::<TmpResult<Vec<vk::ImageView>>>()?;

    let image_count = images.len();
    let command_pools = (0..image_count)
        .map(|_| create_command_pool(device, renderer.queue_family_indices.graphics_family))
        .collect::<TmpResult<Vec<vk::CommandPool>>>()?;

    let tmp = command_pools
        .iter()
        .map(|&command_pool| allocate_command_buffers(device, command_pool, 1))
        .collect::<TmpResult<Vec<Vec<vk::CommandBuffer>>>>()?;
    let command_buffers = tmp.into_iter().flat_map(|cb| cb).collect();

    Ok((
        surface,
        swapchain,
        surface_format.format,
        surface_format.color_space,
        surface_resolution,
        present_mode,
        images,
        image_views,
        command_pools,
        command_buffers,
    ))
}

fn choose_swapchain_format(
    surface_loader: &ash::extensions::khr::Surface,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> TmpResult<vk::SurfaceFormatKHR> {
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
    surface_loader: &ash::extensions::khr::Surface,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> TmpResult<vk::PresentModeKHR> {
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

fn create_command_pool(device: &Device, queue_family_index: u32) -> TmpResult<vk::CommandPool> {
    let command_pool_create_info =
        vk::CommandPoolCreateInfo::builder().queue_family_index(queue_family_index);
    let command_pool = unsafe { device.create_command_pool(&command_pool_create_info, None)? };
    Ok(command_pool)
}

fn allocate_command_buffers(
    device: &Device,
    command_pool: vk::CommandPool,
    count: u32,
) -> TmpResult<Vec<vk::CommandBuffer>> {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(count);

    let command_buffers =
        unsafe { device.allocate_command_buffers(&command_buffer_allocate_info)? };

    Ok(command_buffers)
}
