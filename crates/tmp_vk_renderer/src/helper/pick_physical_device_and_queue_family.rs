use ash::{vk, Entry, Instance};

use super::{QueueFamilyIndices, TmpResult};

pub fn pick_physical_device_and_queue_family(
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
