use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    ffi::{c_char, CString},
    rc::Rc,
};

use ash::vk;
use raw_window_handle::RawDisplayHandle;

pub(crate) enum VulkanObject {
    Image(vk::Image),
    ImageView(vk::ImageView),
    Surface(vk::SurfaceKHR),
    Swapchain(vk::SwapchainKHR),
    Semaphore(vk::Semaphore),
    Fence(vk::Fence),
    CommandPool(vk::CommandPool),
    RenderPass(vk::RenderPass),
    Framebuffer(vk::Framebuffer),
}

pub struct Device {
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) instance_info: vk::InstanceCreateInfo,
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) device: ash::Device,
    pub(crate) device_info: vk::DeviceCreateInfo,
    pub(crate) surface_loader: ash::extensions::khr::Surface,
    pub(crate) swapchain_loader: ash::extensions::khr::Swapchain,
    pub(crate) graphics_queue_family_index: u32,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) debug_utils_loader: ash::extensions::ext::DebugUtils,
    pub(crate) debug_messenger_create_info: vk::DebugUtilsMessengerCreateInfoEXT,
    pub(crate) debug_messenger: vk::DebugUtilsMessengerEXT,

    dropped_object_queue_index: Cell<usize>,
    dropped_object_queues: RefCell<[VecDeque<VulkanObject>; 2]>,
}

impl Device {
    pub fn new(display_handle: &RawDisplayHandle) -> Self {
        unsafe {
            let entry = ash::Entry::load().unwrap();
            let (instance, instance_create_info) =
                create_instance(&entry, display_handle).expect("create_instance failed.");
            let physical_device = pick_physical_device(&instance)
                .expect("pick_physical_device can't find physical device.");

            let graphics_queue_family_index = instance
                .get_physical_device_queue_family_properties(physical_device)
                .iter()
                .enumerate()
                .find_map(|(index, prop)| {
                    if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                        Some(index as u32)
                    } else {
                        None
                    }
                })
                .expect("can't find graphics queue family.");

            let (device, device_create_info) =
                create_device(&instance, &physical_device, graphics_queue_family_index)
                    .expect("create_device failed.");

            let graphics_queue = device.get_device_queue(graphics_queue_family_index, 0);

            let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
            let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);

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
            let debug_messenger = debug_utils_loader
                .create_debug_utils_messenger(&debug_messenger_create_info, None)
                .unwrap();

            let dropped_object_queues = Default::default();

            Device {
                entry,
                instance,
                instance_info: instance_create_info,
                physical_device,
                device,
                device_info: device_create_info,
                surface_loader,
                swapchain_loader,
                graphics_queue_family_index,
                graphics_queue,
                debug_utils_loader,
                debug_messenger,
                debug_messenger_create_info,
                dropped_object_queues,
                dropped_object_queue_index: Cell::new(0),
            }
        }
    }

    pub(crate) fn push_dropped_object(self: &Rc<Self>, object: VulkanObject) {
        let queue =
            &mut self.dropped_object_queues.borrow_mut()[self.dropped_object_queue_index.get()];
        queue.push_back(object)
    }

    pub fn destroy_dropped_objects(&self) {
        let mut index = self.dropped_object_queue_index.get();
        let queues = &mut self.dropped_object_queues.borrow_mut();
        let objects = &mut queues[index];
        if !objects.is_empty() {
            unsafe { self.device.device_wait_idle().unwrap() };
            while !objects.is_empty() {
                let object = objects.pop_front().unwrap();
                unsafe {
                    match object {
                        VulkanObject::Image(image) => self.device.destroy_image(image, None),
                        VulkanObject::ImageView(view) => self.device.destroy_image_view(view, None),
                        VulkanObject::Surface(surface) => {
                            self.surface_loader.destroy_surface(surface, None)
                        }
                        VulkanObject::Swapchain(swapchain) => {
                            self.swapchain_loader.destroy_swapchain(swapchain, None)
                        }
                        VulkanObject::Semaphore(semaphore) => {
                            self.device.destroy_semaphore(semaphore, None)
                        }
                        VulkanObject::Fence(fence) => self.device.destroy_fence(fence, None),
                        VulkanObject::CommandPool(pool) => {
                            self.device.destroy_command_pool(pool, None)
                        }
                        VulkanObject::RenderPass(pass) => {
                            self.device.destroy_render_pass(pass, None)
                        }
                        VulkanObject::Framebuffer(fb) => self.device.destroy_framebuffer(fb, None),
                    }
                }
            }
        }

        index = (index + 1) % queues.len();

        self.dropped_object_queue_index.set(index);
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("device_wait_idle failed.");
            let queue_len = self.dropped_object_queues.borrow().len();
            for _ in 0..queue_len {
                self.destroy_dropped_objects();
            }
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_messenger, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn create_instance(
    entry: &ash::Entry,
    display_handle: &RawDisplayHandle,
) -> ash::prelude::VkResult<(ash::Instance, vk::InstanceCreateInfo)> {
    unsafe {
        let app_name = CString::new("tempura").unwrap();
        let engine_name = CString::new("tempura").unwrap();

        let appinfo = vk::ApplicationInfo::builder()
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
        extension_names.push(ash::extensions::ext::DebugUtils::name().as_ptr());
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
            .application_info(&appinfo)
            .enabled_extension_names(&extension_names)
            .flags(create_flags);
        let create_info = if cfg!(any(feature = "develop", feature = "debug")) {
            create_info.enabled_layer_names(&layer_names)
        } else {
            create_info
        };
        match entry.create_instance(&create_info, None) {
            Ok(instance) => Ok((instance, *create_info)),
            Err(err) => Err(err),
        }
    }
}

/// Pick PhysicalDevice.
/// The device that has a graphic cue is picked. Also, DISCRETE_GPU type is preferred.
fn pick_physical_device(instance: &ash::Instance) -> Option<vk::PhysicalDevice> {
    unsafe {
        let pdevices = instance
            .enumerate_physical_devices()
            .expect("enumerate physical devices error");
        let filtered = pdevices
            .iter()
            .filter_map(|pdevice| {
                if instance
                    .get_physical_device_queue_family_properties(*pdevice)
                    .iter()
                    .any(|info| info.queue_flags.contains(vk::QueueFlags::GRAPHICS))
                {
                    Some(*pdevice)
                } else {
                    None
                }
            })
            .collect::<Vec<vk::PhysicalDevice>>();
        let discrete = filtered.iter().find(|pdevice| {
            instance
                .get_physical_device_properties(**pdevice)
                .device_type
                == vk::PhysicalDeviceType::DISCRETE_GPU
        });
        if let Some(pdevice) = discrete {
            Some(*pdevice)
        } else if let Some(pdevice) = filtered.first() {
            Some(*pdevice)
        } else {
            None
        }
    }
}

fn create_device(
    instance: &ash::Instance,
    pdevice: &vk::PhysicalDevice,
    graphics_queue_family_index: u32,
) -> ash::prelude::VkResult<(ash::Device, vk::DeviceCreateInfo)> {
    unsafe {
        let extension_names = [
            ash::extensions::khr::Swapchain::name().as_ptr(),
            // #[cfg(any(target_os = "macos", target_os = "ios"))]
            vk::KhrPortabilitySubsetFn::name().as_ptr(),
        ];
        let features = vk::PhysicalDeviceFeatures {
            shader_clip_distance: 1,
            ..Default::default()
        };
        let queue_priorities = [1.0];
        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(graphics_queue_family_index)
            .queue_priorities(&queue_priorities)
            .build();
        let queue_infos = [queue_info];
        let create_info = vk::DeviceCreateInfo::builder()
            .enabled_extension_names(&extension_names)
            .enabled_features(&features)
            .queue_create_infos(&queue_infos)
            .build();
        match instance.create_device(*pdevice, &create_info, None) {
            Ok(device) => Ok((device, create_info)),
            Err(err) => Err(err),
        }
    }
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
