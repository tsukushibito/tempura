use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use crate::{common::*, VkSwapchain};
use ash::{extensions, vk, Device, Entry, Instance};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

/// Represents a Vulkan renderer.
pub struct VkRenderer {
    /// Entry point for Vulkan API functions.
    pub(crate) entry: Entry,
    /// Vulkan instance, representing a connection between the application and the Vulkan library.
    pub(crate) instance: Instance,
    /// Messenger for Vulkan debug utility, helpful for debugging.
    pub(crate) debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    /// Physical device, representing a Vulkan compatible GPU.
    pub(crate) physical_device: vk::PhysicalDevice,
    /// Indices for the queue families on the physical device.
    pub(crate) queue_family_indices: QueueFamilyIndices,
    /// Logical device, representing a virtual device built on the physical device.
    pub(crate) device: Device,
    /// Queue for graphics operations.
    graphics_queue: vk::Queue,
    /// Queue for presentation operations.
    present_queue: vk::Queue,
    /// Collection of framebuffers, mapped by their image views.
    framebuffers: RefCell<HashMap<vk::ImageView, vk::Framebuffer>>,

    render_pass: Cell<Option<vk::RenderPass>>,
}

impl VkRenderer {
    /// Constructs a new `VkRenderer`.
    ///
    /// # Arguments
    /// * `display_handle` - Handle to the display.
    /// * `window_handle` - Handle to the window.
    ///
    /// # Returns
    /// A result containing the new `VkRenderer` or an error.
    pub fn new(
        display_handle: &RawDisplayHandle,
        window_handle: &RawWindowHandle,
    ) -> TmpResult<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = create_instance(&entry, display_handle)?;
        let debug_utils_messenger = create_debug_utils_messenger(&entry, &instance)?;
        let dummy_surface = unsafe {
            ash_window::create_surface(&entry, &instance, *display_handle, *window_handle, None)?
        };
        let (physical_device, queue_family_indices) =
            pick_physical_device_and_queue_family(&entry, &instance, &dummy_surface)?;

        let surface_loader = extensions::khr::Surface::new(&entry, &instance);
        unsafe { surface_loader.destroy_surface(dummy_surface, None) };

        let device = create_device(&instance, &physical_device, &queue_family_indices)?;

        let graphics_queue =
            unsafe { device.get_device_queue(queue_family_indices.graphics_family, 0) };
        let present_queue =
            unsafe { device.get_device_queue(queue_family_indices.present_family, 0) };

        Ok(Self {
            entry,
            instance,
            debug_utils_messenger,
            physical_device,
            queue_family_indices,
            device,
            graphics_queue,
            present_queue,
            framebuffers: Default::default(),
            render_pass: Cell::new(None),
        })
    }

    /// Renders using the given swapchain.
    pub fn render(&self, swapchain: &VkSwapchain) -> TmpResult<()> {
        if let Some(_) = self.render_pass.get() {
        } else {
            let render_pass = create_render_pass(&self.device, swapchain.image_format, None)?;
            self.render_pass.set(Some(render_pass));
        }

        swapchain.wait_for_current_frame_fence();

        let (frame_resource, is_suboptimal) = swapchain.acquire_next_frame_resource()?;

        let framebuffer: vk::Framebuffer = *self
            .framebuffers
            .borrow_mut()
            .entry(frame_resource.image_view)
            .or_insert(create_framebuffer(
                &self.device,
                &self.render_pass.get().unwrap(),
                &frame_resource.image_view,
                &swapchain.image_extent,
            )?);

        let command_buffer = &frame_resource.command_buffer;
        let image_available_semaphore = &frame_resource.image_available_semaphore;
        let render_finished_semaphore = &frame_resource.render_finished_semaphore;
        let in_flight_fence = &frame_resource.in_flight_fence;

        // コマンドバッファの開始
        self.begin_command_buffer(command_buffer)?;

        // クリア操作の記録
        self.record_clear_command(*command_buffer, framebuffer, swapchain.image_extent);

        // コマンドバッファの終了
        self.end_command_buffer(command_buffer)?;

        // コマンドバッファをキューにサブミット
        let command_buffers = [*command_buffer];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[*image_available_semaphore])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers)
            .signal_semaphores(&[*render_finished_semaphore])
            .build();

        unsafe {
            self.device
                .queue_submit(self.graphics_queue, &[submit_info], *in_flight_fence)?;
        }

        swapchain.present(self.present_queue, frame_resource.render_finished_semaphore)?;

        Ok(())
    }

    pub(crate) fn release_framebuffer(&mut self, image_view: &vk::ImageView) {
        self.framebuffers.borrow_mut().remove(image_view);
    }

    fn begin_command_buffer(&self, command_buffer: &vk::CommandBuffer) -> TmpResult<()> {
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::SIMULTANEOUS_USE); // 通常はSIMULTANEOUS_USEを指定

        unsafe {
            self.device
                .begin_command_buffer(*command_buffer, &begin_info)?
        };
        Ok(())
    }

    fn record_clear_command(
        &self,
        command_buffer: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        extent: vk::Extent2D,
    ) {
        let clear_color = vk::ClearColorValue {
            float32: [0.0, 0.5, 0.5, 1.0], // クリアする色（ここでは黒）
        };

        let clear_values = [vk::ClearValue { color: clear_color }];
        let render_area = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.render_pass.get().unwrap()) // 適切なレンダーパスを指定
            .framebuffer(framebuffer) // 適切なフレームバッファを指定
            .render_area(render_area)
            .clear_values(&clear_values);

        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            // ここで追加のレンダリングコマンドを記録できます。

            self.device.cmd_end_render_pass(command_buffer);
        }
    }

    fn end_command_buffer(&self, command_buffer: &vk::CommandBuffer) -> TmpResult<()> {
        unsafe {
            self.device.end_command_buffer(*command_buffer)?;
        }
        Ok(())
    }

    fn submit_command_buffer(&self, command_buffer: vk::CommandBuffer, swapchain: &VkSwapchain) {
        // コマンドバッファをキューにサブミットするロジック
        todo!();
    }
}

impl Drop for VkRenderer {
    /// Cleans up Vulkan resources when the `VkRenderer` is dropped.
    fn drop(&mut self) {
        _ = unsafe { self.device.device_wait_idle() };
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(&self.entry, &self.instance);
        unsafe {
            debug_utils_loader.destroy_debug_utils_messenger(self.debug_utils_messenger, None)
        };
        unsafe { self.device.destroy_device(None) };
        unsafe { self.instance.destroy_instance(None) };
    }
}

/// Creates a Vulkan instance.
///
/// # Arguments
/// * `entry` - Reference to the Vulkan entry point.
/// * `display_handle` - Handle to the display.
///
/// # Returns
/// A result containing the created Vulkan instance or an error.
fn create_instance(entry: &Entry, display_handle: &RawDisplayHandle) -> TmpResult<Instance> {
    let app_name = CString::new("tempura")?;
    let engine_name = CString::new("tempura")?;

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::make_api_version(0, 1, 3, 0));

    let mut layer_properties = entry.enumerate_instance_layer_properties()?;
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

    for &c_str_ptr in &layer_names {
        // 安全性のためにnullチェックを行う
        if !c_str_ptr.is_null() {
            // C文字列をRustのCStr型に変換
            let c_str = unsafe { CStr::from_ptr(c_str_ptr) };

            // CStr型をRustの文字列スライスに変換
            if let Ok(str_slice) = c_str.to_str() {
                // ログ出力
                println!("Layer Name: {}", str_slice);
            }
        }
    }

    let mut extension_names = ash_window::enumerate_required_extensions(*display_handle)?.to_vec();
    extension_names.push(extensions::ext::DebugUtils::name().as_ptr());

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

/// Creates a Vulkan debug utils messenger.
///
/// # Arguments
/// * `entry` - Reference to the Vulkan entry point.
/// * `instance` - Reference to the Vulkan instance.
///
/// # Returns
/// A result containing the debug utils messenger or an error.
fn create_debug_utils_messenger(
    entry: &Entry,
    instance: &Instance,
) -> TmpResult<vk::DebugUtilsMessengerEXT> {
    let debug_utils_loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
    let debug_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
        )
        .pfn_user_callback(Some(debug_callback))
        .build();
    let debug_messenger = unsafe {
        debug_utils_loader.create_debug_utils_messenger(&debug_messenger_create_info, None)?
    };

    Ok(debug_messenger)
}

/// Debug callback function for Vulkan.
///
/// # Arguments
/// * `message_severity` - The severity of the debug message.
/// * `message_type` - The type of the debug message.
/// * `p_callback_data` - Pointer to the callback data.
/// * `_user_data` - User data pointer.
///
/// # Returns
/// A boolean value according to the Vulkan API.
unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        std::borrow::Cow::from("")
    } else {
        std::ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        std::borrow::Cow::from("")
    } else {
        std::ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}\n",
        message_severity, message_type, message_id_name, message_id_number, message,
    );

    vk::FALSE
}

/// Picks a physical device and its queue family.
///
/// # Arguments
/// * `entry` - Reference to the Vulkan entry point.
/// * `instance` - Reference to the Vulkan instance.
/// * `surface` - Reference to the Vulkan surface.
///
/// # Returns
/// A result containing the physical device and its queue family indices or an error.
fn pick_physical_device_and_queue_family(
    entry: &Entry,
    instance: &Instance,
    surface: &vk::SurfaceKHR,
) -> TmpResult<(vk::PhysicalDevice, QueueFamilyIndices)> {
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

/// Finds queue family indices for a given physical device.
///
/// # Arguments
/// * `entry` - Reference to the Vulkan entry point.
/// * `instance` - Reference to the Vulkan instance.
/// * `physical_device` - The physical device.
/// * `surface` - Reference to the Vulkan surface.
///
/// # Returns
/// An option containing the queue family indices or None.
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

    let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

    for (index, queue_family) in queue_families.iter().enumerate() {
        if graphics_family.is_none() && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            graphics_family = Some(index as u32);
        }

        let is_present_supported = unsafe {
            surface_loader
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
            graphics_family: graphics_family.unwrap(),
            present_family: present_family.unwrap(),
        })
    } else {
        None
    }
}

/// Creates a Vulkan logical device.
///
/// # Arguments
/// * `instance` - Reference to the Vulkan instance.
/// * `physical_device` - Reference to the physical device.
/// * `queue_family_indices` - Queue family indices.
///
/// # Returns
/// A result containing the created device or an error.
fn create_device(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    queue_family_indices: &QueueFamilyIndices,
) -> TmpResult<Device> {
    let extension_names = [
        ash::extensions::khr::Swapchain::name().as_ptr(),
        // #[cfg(any(target_os = "macos", target_os = "ios"))]
        vk::KhrPortabilitySubsetFn::name().as_ptr(),
    ];

    let queue_priorities = [1.0];
    let graphics_family_index = queue_family_indices.graphics_family;
    let graphics_queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(graphics_family_index)
        .queue_priorities(&queue_priorities)
        .build();
    let mut queue_infos = vec![graphics_queue_create_info];

    let present_family_index = queue_family_indices.present_family;
    if present_family_index != graphics_family_index {
        let present_queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_indices.present_family)
            .queue_priorities(&queue_priorities)
            .build();
        queue_infos.push(present_queue_create_info);
    }

    let create_info = vk::DeviceCreateInfo::builder()
        .enabled_extension_names(&extension_names)
        .queue_create_infos(&queue_infos)
        .build();

    let device = unsafe { instance.create_device(*physical_device, &create_info, None)? };
    Ok(device)
}

/// Creates a Vulkan framebuffer.
///
/// # Arguments
/// * `device` - Reference to the Vulkan logical device.
/// * `render_pass` - The render pass with which the framebuffer is compatible.
/// * `image_view` - The image view to be bound to the framebuffer.
/// * `extent` - The width and height of the framebuffer.
///
/// # Returns
/// A result containing the created framebuffer or an error.
fn create_framebuffer(
    device: &Device,
    render_pass: &vk::RenderPass,
    image_view: &vk::ImageView,
    extent: &vk::Extent2D,
) -> TmpResult<vk::Framebuffer> {
    let attachments = [*image_view];

    let framebuffer_info = vk::FramebufferCreateInfo::builder()
        .render_pass(*render_pass)
        .attachments(&attachments)
        .width(extent.width)
        .height(extent.height)
        .layers(1);

    let framebuffer = unsafe { device.create_framebuffer(&framebuffer_info, None)? };
    Ok(framebuffer)
}

/// Creates a Vulkan render pass.
///
/// This function is responsible for setting up a render pass in Vulkan, which defines how the
/// images in the framebuffer will be used throughout the rendering operations.
///
/// # Arguments
/// * `device` - Reference to the Vulkan logical device.
/// * `color_format` - The format of the color image in the framebuffer.
/// * `depth_format` - Optional format of the depth image in the framebuffer.
///
/// # Returns
/// A result containing the created `vk::RenderPass` or an error if the render pass creation fails.
///
/// # Example
/// ```
/// // Example usage
/// let render_pass = create_render_pass(&device, color_format, Some(depth_format));
/// ```
fn create_render_pass(
    device: &Device,
    color_format: vk::Format,
    depth_format: Option<vk::Format>,
) -> TmpResult<vk::RenderPass> {
    let color_attachment = vk::AttachmentDescription::builder()
        .format(color_format)
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

    let mut attachments = vec![color_attachment];
    let mut subpass_description = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_attachment_ref));

    let depth_attachment_ref;

    if let Some(depth_format) = depth_format {
        let depth_attachment = vk::AttachmentDescription::builder()
            .format(depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        attachments.push(depth_attachment);
        subpass_description = subpass_description.depth_stencil_attachment(&depth_attachment_ref);
    }

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(&attachments)
        .subpasses(std::slice::from_ref(&subpass_description.build()))
        .build();

    let render_pass = unsafe { device.create_render_pass(&render_pass_info, None)? };

    Ok(render_pass)
}
