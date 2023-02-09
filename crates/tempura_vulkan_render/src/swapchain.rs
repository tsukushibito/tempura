use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use ash::{prelude::VkResult, vk};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};
use tempura_render as tr;

use crate::Device;

pub struct Swapchain {
    device: Rc<Device>,
    window_size_provider: Rc<dyn tr::WindowSizeProvider>,
    surface: vk::SurfaceKHR,
    swapchain: Cell<vk::SwapchainKHR>,
    swapchain_info: Cell<vk::SwapchainCreateInfoKHR>,
    image_views: RefCell<Vec<vk::ImageView>>,
    render_pass: Cell<vk::RenderPass>,
    framebuffers: RefCell<Vec<vk::Framebuffer>>,

    // render_targets: RefCell<Vec<RenderTarget>>,
    next_image_index: Cell<u32>,
}

impl Swapchain {
    pub(crate) fn new(
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

            let (swapchain, swapchain_info, present_image_views, render_pass, framebuffers) =
                create_swapchain_objects(
                    window_size_provider.as_ref(),
                    &device.physical_device,
                    &device.device,
                    &device.swapchain_loader,
                    &device.surface_loader,
                    &surface,
                );

            Swapchain {
                device: device.clone(),
                window_size_provider: window_size_provider.clone(),
                surface,
                swapchain: Cell::new(swapchain),
                swapchain_info: Cell::new(swapchain_info),
                image_views: RefCell::new(present_image_views),
                render_pass: Cell::new(render_pass),
                framebuffers: RefCell::new(framebuffers),
                next_image_index: Cell::new(0),
            }
        }
    }

    pub(crate) fn acquire_next_image(&self, semaphore: &vk::Semaphore) -> bool {
        unsafe {
            match self.device.swapchain_loader.acquire_next_image(
                self.swapchain.get(),
                std::u64::MAX,
                *semaphore,
                vk::Fence::null(),
            ) {
                Ok(r) => {
                    self.next_image_index.set(r.0);
                    true
                }
                Err(r)
                    if r == vk::Result::ERROR_OUT_OF_DATE_KHR
                        || r == vk::Result::SUBOPTIMAL_KHR =>
                {
                    // println!("Need to recreate swapchain");
                    self.recreate_swapchain_resources();
                    false
                }
                Err(r) => panic!("acquire_next_image error. {}", r),
            }
        }
    }

    pub(crate) fn begin_render_pass(
        &self,
        clear_values: &[vk::ClearValue],
        command_buffer: &vk::CommandBuffer,
    ) {
        unsafe {
            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(self.render_pass.get())
                .framebuffer(self.framebuffers.borrow()[self.next_image_index.get() as usize])
                .render_area(self.swapchain_info.get().image_extent.into())
                .clear_values(&clear_values)
                .build();

            self.device.device.cmd_begin_render_pass(
                *command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }
    }

    pub(crate) fn end_render_pass(&self, command_buffer: &vk::CommandBuffer) {
        unsafe {
            self.device.device.cmd_end_render_pass(*command_buffer);
        }
    }

    pub(crate) fn present(&self, semaphore: &vk::Semaphore, queue: &vk::Queue) -> VkResult<bool> {
        unsafe {
            let present_info = vk::PresentInfoKHR::builder()
                .swapchains(&[self.swapchain.get()])
                .wait_semaphores(&[*semaphore])
                .image_indices(&[self.next_image_index.get()])
                .build();

            match self
                .device
                .swapchain_loader
                .queue_present(*queue, &present_info)
            {
                Ok(r) => Ok(r),
                Err(r)
                    if r == vk::Result::ERROR_OUT_OF_DATE_KHR
                        || r == vk::Result::SUBOPTIMAL_KHR =>
                {
                    // println!("Need to recreate swapchain");
                    self.recreate_swapchain_resources();
                    Ok(false)
                }
                Err(r) => Err(r),
            }
        }
    }

    fn destory_swapchain_resources(&self) {
        unsafe {
            self.device.device.device_wait_idle().unwrap();
            self.device
                .device
                .destroy_render_pass(self.render_pass.get(), None);
            self.framebuffers
                .borrow()
                .iter()
                .for_each(|&framebuffer| self.device.device.destroy_framebuffer(framebuffer, None));
            self.image_views
                .borrow()
                .iter()
                .for_each(|&view| self.device.device.destroy_image_view(view, None));
            self.device
                .swapchain_loader
                .destroy_swapchain(self.swapchain.get(), None);
        }
    }

    fn recreate_swapchain_resources(&self) {
        self.destory_swapchain_resources();
        let (swapchain, swapchain_info, present_image_views, render_pass, framebuffers) =
            create_swapchain_objects(
                self.window_size_provider.as_ref(),
                &self.device.physical_device,
                &self.device.device,
                &self.device.swapchain_loader,
                &self.device.surface_loader,
                &self.surface,
            );

        self.swapchain.set(swapchain);
        self.swapchain_info.set(swapchain_info);
        *(self.image_views.borrow_mut()) = present_image_views;
        self.render_pass.set(render_pass);
        *(self.framebuffers.borrow_mut()) = framebuffers;
    }
}

fn create_swapchain_objects(
    window_size_provider: &dyn tr::WindowSizeProvider,
    physical_device: &ash::vk::PhysicalDevice,
    device: &ash::Device,
    swapchain_loader: &ash::extensions::khr::Swapchain,
    surface_loader: &ash::extensions::khr::Surface,
    surface: &vk::SurfaceKHR,
) -> (
    vk::SwapchainKHR,
    vk::SwapchainCreateInfoKHR,
    Vec<vk::ImageView>,
    vk::RenderPass,
    Vec<vk::Framebuffer>,
) {
    unsafe {
        let (width, height) = window_size_provider.window_size();
        let extent = vk::Extent2D { width, height };

        let surface_formats = surface_loader
            .get_physical_device_surface_formats(*physical_device, *surface)
            .unwrap();
        let surface_format = *surface_formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                    && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&surface_formats[0]);
        let surface_capabilities = surface_loader
            .get_physical_device_surface_capabilities(*physical_device, *surface)
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
        let present_mode = surface_loader
            .get_physical_device_surface_present_modes(*physical_device, *surface)
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
        let swapchain = swapchain_loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("create_swapchain failed.");

        let present_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
        let present_image_views = present_images
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
                device
                    .create_image_view(&info, None)
                    .expect("Create image view error.")
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

        let create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&[color_attachment_desc])
            .subpasses(&[subpass_desc])
            .build();

        let render_pass = device
            .create_render_pass(&create_info, None)
            .expect("create_render_pass failed.");

        let framebuffers = present_image_views
            .iter()
            .map(|&view| {
                let create_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass)
                    .attachments(&[view])
                    .width(extent.width)
                    .height(extent.height)
                    .layers(1)
                    .build();
                device
                    .create_framebuffer(&create_info, None)
                    .expect("Create framaebuffer error.")
            })
            .collect::<Vec<vk::Framebuffer>>();

        (
            swapchain,
            swapchain_create_info,
            present_image_views,
            render_pass,
            framebuffers,
        )
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        let swapchain = self.swapchain.get();
        let render_pass = self.render_pass.get();
        let framebuffers = self.framebuffers.borrow().to_vec();
        let views = self.image_views.borrow().to_vec();
        let surface = self.surface;
        self.device.request_destroy(Box::new(move |context| unsafe {
            context.device.destroy_render_pass(render_pass, None);
            framebuffers
                .iter()
                .for_each(|&framebuffer| context.device.destroy_framebuffer(framebuffer, None));
            views
                .iter()
                .for_each(|&view| context.device.destroy_image_view(view, None));
            context.swapchain_loader.destroy_swapchain(swapchain, None);
            context.surface_loader.destroy_surface(surface, None);
        }));
    }
}
