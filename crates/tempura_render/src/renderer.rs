use std::ffi::{c_char, CString};

use ash::{
    extensions::{self, ext::DebugUtils},
    vk, Device, Entry, Instance,
};
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

pub struct QueueFamilyIndices {
    graphics_family: Option<u32>,
    present_family: Option<u32>,
}
struct Swapchain {
    surface: vk::SurfaceKHR,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    framebuffers: Vec<vk::Framebuffer>,
    extent: vk::Extent2D,
    format: vk::Format,
}

pub struct Renderer {
    entry: Entry,
    instance: Instance,
    device: Device,
    physical_device: vk::PhysicalDevice,
    queue_family_indices: QueueFamilyIndices,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: Swapchain,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
}

pub trait Window: HasRawWindowHandle + HasRawDisplayHandle {
    fn window_size(&self) -> (u32, u32);
}

impl Renderer {
    pub fn new<T: Window>(window: &T) -> Result<Self, Box<dyn std::error::Error>> {
        let entry = unsafe { Entry::load()? };
        let instance = create_instance(&entry, &window.raw_display_handle())?;
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        let (physical_device, queue_family_indices) =
            pick_physical_device_and_queue_family(&entry, &instance, &surface)?;
        let device = create_device(&instance, &physical_device, &queue_family_indices)?;
        let (graphics_queue, present_queue) = get_device_queues(&device, &queue_family_indices);
        let swapchain = create_swapchain(
            &entry,
            &instance,
            &device,
            &physical_device,
            &surface,
            &queue_family_indices,
            window,
        )?;
        let command_pool =
            Self::create_command_pool(&device, queue_family_indices.graphics_family.unwrap())?;
        let command_buffers =
            Self::allocate_command_buffers(&device, command_pool, swapchain.framebuffers.len())?;

        Ok(Self {
            instance,
            device,
            physical_device,
            queue_family_indices,
            graphics_queue,
            present_queue,
            swapchain,
            command_pool,
            command_buffers,
            // その他のリソース
        })
    }
}

fn create_instance(
    entry: &Entry,
    display_handle: &RawDisplayHandle,
) -> Result<Instance, Box<dyn std::error::Error>> {
    let app_name = CString::new("tempura")?;
    let engine_name = CString::new("tempura")?;

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(0)
        .engine_name(&engine_name)
        .engine_version(0)
        .api_version(vk::make_api_version(0, 1, 3, 0));

    let mut layer_properties = entry
        .enumerate_instance_layer_properties()
        .expect("enumerate instance layer properties error");
    layer_properties.retain(|&prop| {
        let name = prop
            .layer_name
            .iter()
            .map(|&c| c as u8)
            .collect::<Vec<u8>>();
        !std::str::from_utf8(&name).unwrap().contains("VK_LAYER_EOS")
    });
    #[cfg(not(feature = "debug"))]
    {
        layer_properties.retain(|&prop| {
            let name = prop
                .layer_name
                .iter()
                .map(|&c| c as u8)
                .collect::<Vec<u8>>();
            !std::str::from_utf8(&name)
                .unwrap()
                .contains("VK_LAYER_LUNARG_api_dump")
        });
    }
    let layer_names = layer_properties
        .iter()
        .filter_map(|p| {
            if vk::api_version_major(p.spec_version) == 1
                && vk::api_version_minor(p.spec_version) == 3
            {
                Some(p.layer_name.as_ptr())
            } else {
                None
            }
        })
        .collect::<Vec<*const c_char>>();
    let mut extension_names = ash_window::enumerate_required_extensions(*display_handle)
        .expect("enumerate required extensions error")
        .to_vec();
    extension_names.push(DebugUtils::name().as_ptr());
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        extension_names.push(vk::KhrPortabilityEnumerationFn::name().as_ptr());
        // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
        extension_names.push(vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr());
    }

    let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
        vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        vk::InstanceCreateFlags::default()
    };

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names)
        .flags(create_flags);
    let create_info = if cfg!(any(feature = "develop", feature = "debug")) {
        create_info.enabled_layer_names(&layer_names)
    } else {
        create_info
    };

    let instance = unsafe { entry.create_instance(&create_info, None)? };

    Ok(instance)
}

fn pick_physical_device_and_queue_family(
    entry: &Entry,
    instance: &Instance,
    surface: &vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, QueueFamilyIndices), Box<dyn std::error::Error>> {
    let physical_devices = unsafe { instance.enumerate_physical_devices()? };
    if physical_devices.is_empty() {
        return Err("No Vulkan-compatible devices found".into());
    }

    for &physical_device in &physical_devices {
        if let Some(queue_family_indices) =
            find_queue_family_indices(entry, instance, physical_device, surface)
        {
            return Ok((physical_device, queue_family_indices));
        }
    }

    Err("No suitable physical device found".into())
}

fn find_queue_family_indices(
    entry: &Entry,
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> Option<QueueFamilyIndices> {
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    let mut graphics_family = None;
    let mut present_family = None;

    let surface_entry = extensions::khr::Surface::new(entry, instance);

    for (index, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            graphics_family = Some(index as u32);
        }

        let is_present_supported = unsafe {
            surface_entry
                .get_physical_device_surface_support(physical_device, index as u32, *surface)
                .unwrap()
        };
        if is_present_supported {
            present_family = Some(index as u32);
        }

        if graphics_family.is_some() && present_family.is_some() {
            break;
        }
    }

    if graphics_family.is_some() && present_family.is_some() {
        Some(QueueFamilyIndices {
            graphics_family,
            present_family,
        })
    } else {
        None
    }
}

fn create_device(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    queue_family_indices: &QueueFamilyIndices,
) -> Result<ash::Device, Box<dyn std::error::Error>> {
    let extension_names = [
        ash::extensions::khr::Swapchain::name().as_ptr(),
        // #[cfg(any(target_os = "macos", target_os = "ios"))]
        vk::KhrPortabilitySubsetFn::name().as_ptr(),
    ];

    let queue_priorities = [1.0];
    let graphics_queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics_family.unwrap())
        .queue_priorities(&queue_priorities)
        .build();

    let present_queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_indices.present_family.unwrap())
        .queue_priorities(&queue_priorities)
        .build();

    let queue_infos = [graphics_queue_create_info, present_queue_create_info];
    let create_info = vk::DeviceCreateInfo::builder()
        .enabled_extension_names(&extension_names)
        .queue_create_infos(&queue_infos)
        .build();

    let device = unsafe { instance.create_device(*physical_device, &create_info, None)? };
    Ok(device)
}

fn get_device_queues(
    device: &Device,
    queue_family_indices: &QueueFamilyIndices,
) -> (vk::Queue, vk::Queue) {
    let graphics_queue =
        unsafe { device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0) };

    let present_queue =
        unsafe { device.get_device_queue(queue_family_indices.present_family.unwrap(), 0) };

    (graphics_queue, present_queue)
}

fn create_swapchain<T: Window>(
    entry: &Entry,
    instance: &Instance,
    device: &Device,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
    queue_family_indices: &QueueFamilyIndices,
    window: &T,
) -> Result<Swapchain, Box<dyn std::error::Error>> {
    let surface_entry = extensions::khr::Surface::new(entry, instance);
    let surface_format = choose_swapchain_format(&surface_entry, physical_device, surface)?;

    let present_mode = choose_swapchain_present_mode(&surface_entry, physical_device, surface)?;

    let surface_capabilities = unsafe {
        surface_entry.get_physical_device_surface_capabilities(*physical_device, *surface)?
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

    let queue_family_indices = [
        queue_family_indices.graphics_family.unwrap(),
        queue_family_indices.present_family.unwrap(),
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

    let swapchain_entry = ash::extensions::khr::Swapchain::new(instance, device);
    let swapchain = unsafe { swapchain_entry.create_swapchain(&swapchain_create_info, None)? };
    let images = unsafe { swapchain_entry.get_swapchain_images(swapchain)? };
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

    Ok(Swapchain {
        surface: *surface,
        swapchain,
        images,
        image_views,
        framebuffers,
        extent: swapchain_create_info.image_extent,
        format: swapchain_create_info.image_format,
    })
}

fn choose_swapchain_format(
    surface_entry: &extensions::khr::Surface,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> Result<vk::SurfaceFormatKHR, Box<dyn std::error::Error>> {
    let formats =
        unsafe { surface_entry.get_physical_device_surface_formats(*physical_device, *surface)? };

    for format in formats {
        if format.format == vk::Format::B8G8R8A8_SRGB
            && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        {
            return Ok(format);
        }
    }

    Ok(formats[0])
}

fn choose_swapchain_present_mode(
    surface_entry: &extensions::khr::Surface,
    physical_device: &vk::PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> Result<vk::PresentModeKHR, Box<dyn std::error::Error>> {
    let present_modes = unsafe {
        surface_entry.get_physical_device_surface_present_modes(*physical_device, *surface)?
    };

    for mode in present_modes {
        if mode == vk::PresentModeKHR::MAILBOX {
            return Ok(mode);
        }
    }

    Ok(vk::PresentModeKHR::FIFO)
}
