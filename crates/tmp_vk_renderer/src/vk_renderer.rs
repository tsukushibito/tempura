use crate::helper::*;
use ash::{extensions, vk, Device, Entry, Instance};
use raw_window_handle::{HasRawDisplayHandle, RawDisplayHandle, RawWindowHandle};
use std::ffi::{c_char, CString};

pub struct VkRenderer {
    entry: Entry,       // Vulkanエントリポイント
    instance: Instance, // Vulkanインスタンス
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    physical_device: vk::PhysicalDevice, // 物理デバイス
    queue_family_indices: QueueFamilyIndices,
    device: Device,            // 論理デバイス
    graphics_queue: vk::Queue, // グラフィックスキュー
    present_queue: vk::Queue,
    // command_pool: vk::CommandPool,       // コマンドプール
}

impl VkRenderer {
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
        })
    }
}
