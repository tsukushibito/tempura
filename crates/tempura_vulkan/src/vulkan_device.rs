use std::ffi::{c_char, CString};
use std::rc::Rc;

use ash::{extensions, Device};
use ash::{vk, Entry, Instance};
use raw_window_handle::RawDisplayHandle;

use crate::{
    CommandPool, Fence, QueueFamily, QueueFamilyIndices, RcWindow, Result, Semaphore, Swapchain,
};

pub struct VulkanDevice {
    entry: Entry,
    instance: Instance,
    device: Device,
    physical_device: vk::PhysicalDevice,
    queue_family_indices: QueueFamilyIndices,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanDevice {
    pub fn new(window: &RcWindow) -> Result<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = create_instance(&entry, &window.raw_display_handle())?;

        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
        let debug_messenger_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING, // | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
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

        let dummy_surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        let (physical_device, queue_family_indices) =
            pick_physical_device_and_queue_family(&entry, &instance, &dummy_surface)?;
        let surface_loader = extensions::khr::Surface::new(&entry, &instance);
        unsafe { surface_loader.destroy_surface(dummy_surface, None) };

        let device = create_device(&instance, &physical_device, &queue_family_indices)?;
        let (graphics_queue, present_queue) = get_device_queues(&device, &queue_family_indices);
        Ok(Self {
            entry,
            instance,
            device,
            physical_device,
            queue_family_indices,
            graphics_queue,
            present_queue,
            debug_messenger,
        })
    }

    pub fn create_swapchain(self: &Rc<Self>, window: &RcWindow) -> Result<Rc<Swapchain>> {
        let surface = unsafe {
            ash_window::create_surface(
                &self.entry,
                &self.instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )?
        };
        Ok(Rc::new(Swapchain::new(self, window, &surface)?))
    }

    pub fn create_command_pool(
        self: &Rc<Self>,
        queue_family: QueueFamily,
    ) -> Result<Rc<CommandPool>> {
        let queue_family_index = match queue_family {
            QueueFamily::Graphics => self.queue_family_indices.graphics_family,
            QueueFamily::Present => self.queue_family_indices.present_family,
        };
        Ok(Rc::new(CommandPool::new(self, queue_family_index)?))
    }

    pub fn create_fence(self: &Rc<Self>, signaled: bool) -> Result<Rc<Fence>> {
        Ok(Rc::new(Fence::new(self, signaled)?))
    }

    pub fn create_semaphore(self: &Rc<Self>) -> Result<Rc<Semaphore>> {
        Ok(Rc::new(Semaphore::new(self)?))
    }

    pub(crate) fn device(&self) -> &Device {
        &self.device
    }

    pub(crate) fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub(crate) fn queue_family_indices(&self) -> &QueueFamilyIndices {
        &self.queue_family_indices
    }

    pub(crate) fn graphics_queue(&self) -> vk::Queue {
        self.graphics_queue
    }

    pub(crate) fn present_queue(&self) -> vk::Queue {
        self.present_queue
    }

    pub(crate) fn surface_loader(&self) -> ash::extensions::khr::Surface {
        extensions::khr::Surface::new(&self.entry, &self.instance)
    }

    pub(crate) fn swapchain_loader(&self) -> ash::extensions::khr::Swapchain {
        extensions::khr::Swapchain::new(&self.instance, &self.device)
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        _ = unsafe { self.device.device_wait_idle() };
        let debug_utils_loader = extensions::ext::DebugUtils::new(&self.entry, &self.instance);
        unsafe { debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None) };
        unsafe { self.device.destroy_device(None) };
        unsafe { self.instance.destroy_instance(None) };
    }
}

fn create_instance(entry: &Entry, display_handle: &RawDisplayHandle) -> Result<Instance> {
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
        .enabled_layer_names(&layer_names)
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
) -> Result<(vk::PhysicalDevice, QueueFamilyIndices)> {
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

    let surface_loader = extensions::khr::Surface::new(entry, instance);

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

fn create_device(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    queue_family_indices: &QueueFamilyIndices,
) -> Result<Device> {
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

fn get_device_queues(
    device: &Device,
    queue_family_indices: &QueueFamilyIndices,
) -> (vk::Queue, vk::Queue) {
    let graphics_queue =
        unsafe { device.get_device_queue(queue_family_indices.graphics_family, 0) };

    let present_queue = unsafe { device.get_device_queue(queue_family_indices.present_family, 0) };

    (graphics_queue, present_queue)
}

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
