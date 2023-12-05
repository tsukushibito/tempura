use crate::helper::{create_instance, TmpResult};
use ash::{extensions, vk, Device, Entry, Instance};
use raw_window_handle::RawDisplayHandle;
use std::ffi::{c_char, CString};

pub struct VkRenderer {
    entry: Entry, // Vulkanエントリポイント
    instance: Instance, // Vulkanインスタンス
                  // physical_device: vk::PhysicalDevice, // 物理デバイス
                  // device: Device,                      // 論理デバイス
                  // queue_family_index: u32,             // キューファミリーインデックス
                  // graphics_queue: vk::Queue,           // グラフィックスキュー
                  // command_pool: vk::CommandPool,       // コマンドプール
}

impl VkRenderer {
    pub fn new(display_handle: &RawDisplayHandle) -> TmpResult<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = create_instance(&entry, display_handle)?;

        Ok(Self { entry, instance })
    }
}
