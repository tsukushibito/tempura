use std::ffi::{c_char, CString};

use ash::vk;
use raw_window_handle::RawDisplayHandle;

pub struct Device {
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub surface_loader: ash::extensions::khr::Surface,
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub graphics_queue_family_index: u32,
}

impl Device {
    pub fn new(display_handle: &RawDisplayHandle) -> Self {
        unsafe {
            let entry = ash::Entry::load().unwrap();
            let instance =
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

            let device = create_device(&instance, &physical_device, graphics_queue_family_index)
                .expect("create_device failed.");

            let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
            let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);

            Device {
                instance,
                physical_device,
                device,
                surface_loader,
                swapchain_loader,
                graphics_queue_family_index,
            }
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("device_wait_idle failed.");
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn create_instance(
    entry: &ash::Entry,
    display_handle: &RawDisplayHandle,
) -> ash::prelude::VkResult<ash::Instance> {
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
        entry.create_instance(&create_info, None)
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
) -> ash::prelude::VkResult<ash::Device> {
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
        instance.create_device(*pdevice, &create_info, None)
    }
}
