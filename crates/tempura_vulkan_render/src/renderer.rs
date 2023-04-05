use std::{
    ffi::{c_char, CString},
    rc::Rc,
};

use ash::{extensions::ext::DebugUtils, prelude::VkResult, vk, Device, Entry, Instance};
use raw_window_handle::RawDisplayHandle;

use super::{Material, Shader, Swapchain};
use tempura_render as tr;

pub struct Renderer {
    pub(crate) entry: Entry,
    pub(crate) instance: Instance,
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) device: Rc<Device>,
    pub(crate) surface_loader: Rc<ash::extensions::khr::Surface>,
    pub(crate) swapchain_loader: Rc<ash::extensions::khr::Swapchain>,

    present_queue: vk::Queue,
    present_semaphore: vk::Semaphore,
    render_semaphore: vk::Semaphore,
    _graphics_queue_family_index: u32,
    command_pool: vk::CommandPool,
    _setup_command_buffer: vk::CommandBuffer,
    draw_command_buffer: vk::CommandBuffer,
    render_fence: vk::Fence,
    debug_utils_loader: DebugUtils,
    debug_callback: vk::DebugUtilsMessengerEXT,
}

impl Renderer {
    pub fn new(display_handle: &RawDisplayHandle) -> Self {
        let entry = unsafe { Entry::load().expect("Load entry error") };
        let instance = create_instance(&entry, display_handle).expect("Create instance error");
        let debug_utils_loader = DebugUtils::new(&entry, &instance);
        let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
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
            .pfn_user_callback(Some(vulkan_debug_callback))
            .build();
        let debug_callback = unsafe {
            debug_utils_loader
                .create_debug_utils_messenger(&debug_info, None)
                .unwrap()
        };

        let physical_device = pick_physical_device(&instance).expect("Not found physical device");
        let graphics_queue_family_index =
            get_graphics_queue_family_index(&instance, &physical_device)
                .expect("Not found graphics queue");
        let device = create_device(&instance, &physical_device, graphics_queue_family_index)
            .expect("Create device error");
        let device = Rc::new(device);
        let present_queue = unsafe { device.get_device_queue(graphics_queue_family_index, 0) };
        let command_pool = create_command_pool(&device, graphics_queue_family_index)
            .expect("Create command pool error");
        let command_buffers =
            create_command_buffers(&device, &command_pool).expect("Create command buffers error");
        let setup_command_buffer = command_buffers[0];
        let draw_command_buffer = command_buffers[1];
        let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
        let surface_loader = Rc::new(surface_loader);
        let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);
        let swapchain_loader = Rc::new(swapchain_loader);
        let fence_create_info = vk::FenceCreateInfo::builder()
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();
        let render_fence = unsafe {
            device
                .create_fence(&fence_create_info, None)
                .expect("Create fence error")
        };
        let semaphore_create_info = vk::SemaphoreCreateInfo::default();
        let present_semaphore = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Create semaphore error")
        };
        let render_semaphore = unsafe {
            device
                .create_semaphore(&semaphore_create_info, None)
                .expect("Create semaphore error")
        };

        Renderer {
            entry,
            instance,
            debug_utils_loader,
            debug_callback,
            physical_device,
            device,
            surface_loader,
            swapchain_loader,
            present_queue,
            present_semaphore,
            render_semaphore,
            _graphics_queue_family_index: graphics_queue_family_index,
            command_pool,
            _setup_command_buffer: setup_command_buffer,
            draw_command_buffer,
            render_fence,
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
            self.device.destroy_semaphore(self.present_semaphore, None);
            self.device.destroy_semaphore(self.render_semaphore, None);
            self.device.destroy_fence(self.render_fence, None);
            self.device.destroy_command_pool(self.command_pool, None);
            self.debug_utils_loader
                .destroy_debug_utils_messenger(self.debug_callback, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

impl tr::Renderer for Renderer {
    type Swapchain = Swapchain;
    type Shader = Shader;
    type Material = Material;

    fn render(&self, swapchain: &Self::Swapchain) {
        unsafe {
            if !swapchain.acquire_next_image(&self.present_semaphore) {
                return;
            };

            self.device
                .wait_for_fences(&[self.render_fence], true, std::u64::MAX)
                .expect("Wait for fence failed.");
            self.device
                .reset_fences(&[self.render_fence])
                .expect("Reset fences failed.");

            self.device
                .reset_command_buffer(
                    self.draw_command_buffer,
                    vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Reset command buffer failed.");

            let command_buffer_begin_info = vk::CommandBufferBeginInfo::builder()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT)
                .build();

            self.device
                .begin_command_buffer(self.draw_command_buffer, &command_buffer_begin_info)
                .expect("Begin commandbuffer failed.");

            let clear_values = [vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 0.5, 1.0],
                },
            }];

            swapchain.begin_render_pass(&clear_values, &self.draw_command_buffer);

            swapchain.end_render_pass(&self.draw_command_buffer);

            self.device
                .end_command_buffer(self.draw_command_buffer)
                .expect("End commandbuffer failed.");

            let submit_info = vk::SubmitInfo::builder()
                .wait_semaphores(&[self.present_semaphore])
                .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                .command_buffers(&[self.draw_command_buffer])
                .signal_semaphores(&[self.render_semaphore])
                .build();

            self.device
                .queue_submit(self.present_queue, &[submit_info], self.render_fence)
                .expect("Queue submit failed.");

            swapchain
                .present(&self.render_semaphore, &self.present_queue)
                .unwrap();
        }
    }

    fn create_swapchain(
        self: &Rc<Self>,
        display_handle: &RawDisplayHandle,
        window_handle: &raw_window_handle::RawWindowHandle,
        window_size_provider: &Rc<dyn tempura_render::WindowSizeProvider>,
    ) -> Self::Swapchain {
        Swapchain::new(self, display_handle, window_handle, window_size_provider)
    }

    fn create_shader(
        self: &Rc<Self>,
        vertex_shader_code: &Vec<u8>,
        fragment_shader_code: &Vec<u8>,
    ) -> Self::Shader {
        Shader::new(self, vertex_shader_code, fragment_shader_code)
    }

    fn create_material(self: &Rc<Self>, shader: &Rc<Self::Shader>) -> Self::Material {
        Material::new(self, shader)
    }
}

unsafe extern "system" fn vulkan_debug_callback(
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

/// Create Instance.
/// In case of develop feature, Validation layer etc. will be added.
fn create_instance(entry: &Entry, display_handle: &RawDisplayHandle) -> VkResult<Instance> {
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
fn pick_physical_device(instance: &Instance) -> Option<vk::PhysicalDevice> {
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

fn get_graphics_queue_family_index(
    instance: &Instance,
    pdevice: &vk::PhysicalDevice,
) -> Option<u32> {
    unsafe {
        instance
            .get_physical_device_queue_family_properties(*pdevice)
            .iter()
            .enumerate()
            .find_map(|(index, prop)| {
                if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    Some(index as u32)
                } else {
                    None
                }
            })
    }
}

fn create_device(
    instance: &Instance,
    pdevice: &vk::PhysicalDevice,
    graphics_queue_family_index: u32,
) -> VkResult<Device> {
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

fn create_command_pool(device: &Device, queue_family_index: u32) -> VkResult<vk::CommandPool> {
    unsafe {
        let create_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index)
            .build();
        device.create_command_pool(&create_info, None)
    }
}

fn create_command_buffers(
    device: &Device,
    command_pool: &vk::CommandPool,
) -> VkResult<Vec<vk::CommandBuffer>> {
    unsafe {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_buffer_count(2)
            .command_pool(*command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .build();
        device.allocate_command_buffers(&allocate_info)
    }
}
