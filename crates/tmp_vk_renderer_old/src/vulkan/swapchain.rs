use std::rc::Rc;

use ash::{extensions, vk};

use super::{device::Device, image::Image, image_view::ImageView, semaphore::Semaphore};
use crate::{TmpResult, Window};

pub struct Swapchain {
    device: Rc<Device>,
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    image_format: vk::Format,
    image_color_space: vk::ColorSpaceKHR,
    image_extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    images: Vec<Rc<Image>>,
    image_views: Vec<Rc<ImageView>>,
}

struct SwapchainAttributes {
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    image_format: vk::Format,
    image_color_space: vk::ColorSpaceKHR,
    image_extent: vk::Extent2D,
    present_mode: vk::PresentModeKHR,
    images: Vec<Rc<Image>>,
    image_views: Vec<Rc<ImageView>>,
}

impl Swapchain {
    pub fn new<T: Window>(device: &Rc<Device>, window: &T) -> TmpResult<Swapchain> {
        let attributes = create_swapchain(device, window)?;
        Ok(Self {
            device: device.clone(),
            surface: attributes.surface,
            swapchain: attributes.swapchain,
            image_format: attributes.image_format,
            image_color_space: attributes.image_color_space,
            image_extent: attributes.image_extent,
            present_mode: attributes.present_mode,
            images: attributes.images,
            image_views: attributes.image_views,
        })
    }

    pub fn handle(&self) -> vk::SwapchainKHR {
        self.swapchain
    }

    pub fn image_count(&self) -> usize {
        self.images.len()
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

    pub fn image_views(&self) -> Vec<Rc<ImageView>> {
        self.image_views
            .iter()
            .map(|iv| iv.clone())
            .collect::<Vec<Rc<ImageView>>>()
    }

    pub fn acquire_next_image(&self, semaphore: &Semaphore) -> TmpResult<(u32, Rc<ImageView>)> {
        let swapchain_loader = self.device.swapchain_loader();
        let (image_index, _) = unsafe {
            swapchain_loader.acquire_next_image(
                self.swapchain,
                std::u64::MAX,
                semaphore.handle(),
                vk::Fence::null(),
            )?
        };
        Ok((image_index, self.image_views[image_index as usize].clone()))
    }

    pub fn recreate<T: Window>(&mut self, window: &T) -> TmpResult<()> {
        destroy_swapchain_and_surface(&self.device, self.swapchain, self.surface);
        let attributes = create_swapchain(&self.device, window)?;

        self.surface = attributes.surface;
        self.swapchain = attributes.swapchain;
        self.image_format = attributes.image_format;
        self.image_color_space = attributes.image_color_space;
        self.image_extent = attributes.image_extent;
        self.present_mode = attributes.present_mode;
        self.images = attributes.images;
        self.image_views = attributes.image_views;

        Ok(())
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        destroy_swapchain_and_surface(&self.device, self.swapchain, self.surface);
    }
}

fn choose_swapchain_format(
    surface_loader: &extensions::khr::Surface,
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
    surface_loader: &extensions::khr::Surface,
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

fn create_swapchain<T: Window>(device: &Rc<Device>, window: &T) -> TmpResult<SwapchainAttributes> {
    let surface = device.create_surface(window)?;
    let surface_loader = device.surface_loader();
    let physical_device = device.physical_device();

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
        let (width, height) = window.window_size();
        vk::Extent2D { width, height }
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

    let queue_family_indices = device.queue_family_indices();
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

    let swapchain_loader = device.swapchain_loader();
    let swapchain = unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None)? };
    let images = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
    let images = images
        .iter()
        .map(|i| {
            Rc::new(
                Image::new_for_swapchain(
                    device,
                    *i,
                    swapchain_create_info.image_extent,
                    swapchain_create_info.image_format,
                )
                .unwrap(),
            )
        })
        .collect::<Vec<Rc<Image>>>();

    let image_views = images
        .iter()
        .map(|image| {
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
            Rc::new(
                ImageView::new(
                    device,
                    image,
                    view_type,
                    format,
                    components,
                    subresource_range,
                )
                .unwrap(),
            )
        })
        .collect::<Vec<Rc<ImageView>>>();

    Ok(SwapchainAttributes {
        surface: surface,
        swapchain,
        image_extent: surface_resolution,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        images,
        image_views,
        present_mode,
    })
}

fn destroy_swapchain_and_surface(
    device: &Rc<Device>,
    swapchain: vk::SwapchainKHR,
    surface: vk::SurfaceKHR,
) {
    let device_handle = device.handle();
    unsafe {
        device_handle
            .device_wait_idle()
            .expect("device_wait_idle error")
    };

    let swapchain_loader = device.swapchain_loader();
    unsafe { swapchain_loader.destroy_swapchain(swapchain, None) };

    let surface_loader = device.surface_loader();
    unsafe { surface_loader.destroy_surface(surface, None) };
}
