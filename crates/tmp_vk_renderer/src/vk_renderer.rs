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
    pub fn new(display_handle: &RawDisplayHandle) -> Result<Self> {
        let entry = unsafe { Entry::load()? };
        let instance = create_instance(&entry, display_handle)?;

        Ok(Self { entry, instance })
    }
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create_instance(entry: &Entry, display_handle: &RawDisplayHandle) -> Result<Instance> {
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
