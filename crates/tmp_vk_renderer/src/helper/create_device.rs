use ash::{vk, Device, Instance};

use super::{QueueFamilyIndices, TmpResult};

pub fn create_device(
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
