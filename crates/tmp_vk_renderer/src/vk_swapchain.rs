use std::rc::Rc;

use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{helper::TmpResult, VkRenderer};

pub struct VkSwapchain {
    renderer: Rc<VkRenderer>,
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    image_format: vk::Format,
    image_color_space: vk::ColorSpaceKHR,
    image_extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
}

impl VkSwapchain {
    pub fn new(
        renderer: &Rc<VkRenderer>,
        display_handle: &RawDisplayHandle,
        window_handle: &RawWindowHandle,
        window_width: u32,
        window_height: u32,
    ) -> TmpResult<Self> {
        let (
            surface,
            swapchain,
            image_format,
            image_color_space,
            image_extent,
            present_mode,
            images,
            image_views,
        ) = create_swapchain_resources(
            &renderer,
            display_handle,
            window_handle,
            window_width,
            window_height,
        )?;
        Ok(Self {
            renderer: renderer.clone(),
            surface,
            swapchain,
            image_format,
            image_color_space,
            image_extent,
            present_mode,
            images,
            image_views,
        })
    }
}

impl Drop for VkSwapchain {
    fn drop(&mut self) {
        let entry = &self.renderer.entry;
        let instance = &self.renderer.instance;
        let device = &self.renderer.device;

        unsafe { device.device_wait_idle().expect("device_wait_idle error") };

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
        unsafe { swapchain_loader.destroy_swapchain(self.swapchain, None) };

        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        unsafe { surface_loader.destroy_surface(self.surface, None) };
    }
}

fn create_swapchain_resources(
    renderer: &VkRenderer,
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
)> {
    let entry = &renderer.entry;
    let instance = &renderer.instance;
    let surface = unsafe {
        ash_window::create_surface(entry, instance, *display_handle, *window_handle, None)?
    };
    let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
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
    let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);
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

            let image_view = unsafe { device.create_image_view(&image_view_create_info, None) }
                .expect("Error create_image_view");
            image_view
        })
        .collect::<Vec<vk::ImageView>>();

    Ok((
        surface,
        swapchain,
        surface_format.format,
        surface_format.color_space,
        surface_resolution,
        present_mode,
        images,
        image_views,
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
