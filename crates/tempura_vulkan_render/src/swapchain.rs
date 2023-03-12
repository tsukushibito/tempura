use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use ash::vk;
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use tempura_render as tr;

use crate::{Device, RenderTarget, VulkanObject};

pub struct Swapchain {
    device: Rc<Device>,
    window_size_provider: Rc<dyn tr::WindowSizeProvider>,
    surface: vk::SurfaceKHR,
    swapchain: Cell<vk::SwapchainKHR>,
    swapchain_info: Cell<vk::SwapchainCreateInfoKHR>,
    render_targets: RefCell<Vec<Rc<RenderTarget>>>,
    next_render_target_index: Cell<u32>,
}

impl Swapchain {
    pub fn new(
        device: &Rc<Device>,
        display_handle: &RawDisplayHandle,
        window_handle: &RawWindowHandle,
        window_size_provider: &Rc<dyn tr::WindowSizeProvider>,
    ) -> Self {
        unsafe {
            let surface = ash_window::create_surface(
                &device.entry,
                &device.instance,
                *display_handle,
                *window_handle,
                None,
            )
            .expect("Create surface error.");

            let (width, height) = window_size_provider.window_size();
            let (swapchain, swapchain_info, render_targets) =
                create_swapchain_and_render_targets(width, height, device, &surface);

            let image_count = render_targets.len() as u32;
            Swapchain {
                device: device.clone(),
                window_size_provider: window_size_provider.clone(),
                surface,
                swapchain: Cell::new(swapchain),
                swapchain_info: Cell::new(swapchain_info),
                render_targets: RefCell::new(render_targets),
                next_render_target_index: Cell::new(image_count - 1),
            }
        }
    }

    pub fn acquire_next_render_target(&self) -> Option<Rc<RenderTarget>> {
        unsafe {
            let render_targets = self.render_targets.borrow();
            let index = (self.next_render_target_index.get() + 1) % render_targets.len() as u32;
            let render_target = &render_targets[index as usize];
            match self.device.swapchain_loader.acquire_next_image(
                self.swapchain.get(),
                std::u64::MAX,
                render_target.available_semaphore,
                vk::Fence::null(),
            ) {
                Ok(r) => {
                    assert!(r.0 == index);
                    let index = r.0;
                    self.next_render_target_index.set(index);
                    Some(render_target.clone())
                }
                Err(r)
                    if r == vk::Result::ERROR_OUT_OF_DATE_KHR
                        || r == vk::Result::SUBOPTIMAL_KHR =>
                {
                    // println!("Need to recreate swapchain");
                    self.recreate_swapchain_resources();
                    None
                }
                Err(r) => panic!("acquire_next_image error. {}", r),
            }
        }
    }

    pub fn present(&self) {
        unsafe {
            let render_targets = self.render_targets.borrow();
            let index = self.next_render_target_index.get() as usize;
            let render_target = &render_targets[index];
            let present_info = vk::PresentInfoKHR::builder()
                .swapchains(&[self.swapchain.get()])
                .wait_semaphores(&[render_target.render_finished_semaphore])
                .image_indices(&[self.next_render_target_index.get()])
                .build();

            match self
                .device
                .swapchain_loader
                .queue_present(self.device.render_queue, &present_info)
            {
                Ok(_) => (),
                Err(r)
                    if r == vk::Result::ERROR_OUT_OF_DATE_KHR
                        || r == vk::Result::SUBOPTIMAL_KHR =>
                {
                    // println!("Need to recreate swapchain");
                    self.recreate_swapchain_resources();
                }
                Err(r) => panic!("queue_present error. {}", r),
            }
        }
    }

    fn destory_swapchain_resources(&self) {
        unsafe {
            self.device.device.device_wait_idle().unwrap();
            self.device
                .swapchain_loader
                .destroy_swapchain(self.swapchain.get(), None);
        }
    }

    fn recreate_swapchain_resources(&self) {
        self.destory_swapchain_resources();
        let (width, height) = self.window_size_provider.window_size();
        let (swapchain, swapchain_info, render_targets) =
            create_swapchain_and_render_targets(width, height, &self.device, &self.surface);

        self.swapchain.set(swapchain);
        self.swapchain_info.set(swapchain_info);
        *self.render_targets.borrow_mut() = render_targets;
        self.next_render_target_index.set(0)
    }
}

fn create_swapchain(
    width: u32,
    height: u32,
    device: &Device,
    surface: &vk::SurfaceKHR,
) -> (vk::SwapchainKHR, vk::SwapchainCreateInfoKHR) {
    unsafe {
        let extent = vk::Extent2D { width, height };

        let surface_formats = device
            .surface_loader
            .get_physical_device_surface_formats(device.physical_device, *surface)
            .unwrap();
        let surface_format = *surface_formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&surface_formats[0]);
        let surface_capabilities = device
            .surface_loader
            .get_physical_device_surface_capabilities(device.physical_device, *surface)
            .unwrap();
        let desired_image_count = std::cmp::min(
            surface_capabilities.min_image_count + 1,
            surface_capabilities.max_image_count,
        );
        let extent = if surface_capabilities.current_extent.width == std::u32::MAX {
            extent
        } else {
            surface_capabilities.current_extent
        };
        let present_mode = device
            .surface_loader
            .get_physical_device_surface_present_modes(device.physical_device, *surface)
            .unwrap()
            .iter()
            .cloned()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(*surface)
            .min_image_count(desired_image_count)
            .image_color_space(surface_format.color_space)
            .image_format(surface_format.format)
            .image_extent(extent)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(surface_capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .image_array_layers(1)
            .build();
        let swapchain = device
            .swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("create_swapchain failed.");
        (swapchain, swapchain_create_info)
    }
}

fn create_render_targets(
    device: &Rc<Device>,
    swapchain: &vk::SwapchainKHR,
    swapchain_info: &vk::SwapchainCreateInfoKHR,
) -> Vec<Rc<RenderTarget>> {
    unsafe {
        let images = device
            .swapchain_loader
            .get_swapchain_images(*swapchain)
            .expect("get_swapchain_images error.");
        let render_targets = images
            .iter()
            .map(|&image| {
                Rc::new(RenderTarget::new_from_swapchain_image(
                    device,
                    swapchain_info.image_extent,
                    swapchain_info.image_format,
                    image,
                ))
            })
            .collect::<Vec<Rc<RenderTarget>>>();
        render_targets
    }
}

fn create_swapchain_and_render_targets(
    width: u32,
    height: u32,
    device: &Rc<Device>,
    surface: &vk::SurfaceKHR,
) -> (
    vk::SwapchainKHR,
    vk::SwapchainCreateInfoKHR,
    Vec<Rc<RenderTarget>>,
) {
    let (swapchain, swapchain_info) = create_swapchain(width, height, device, surface);
    let render_targets = create_render_targets(device, &swapchain, &swapchain_info);
    (swapchain, swapchain_info, render_targets)
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        self.device
            .push_dropped_object(VulkanObject::Swapchain(self.swapchain.get()));
        self.device
            .push_dropped_object(VulkanObject::Surface(self.surface));
    }
}
