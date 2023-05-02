use std::rc::Rc;

use ash::vk;

use crate::{Device, Swapchain, TvResult};

pub struct Image {
    device: Rc<Device>,
    image: vk::Image,
    memory: vk::DeviceMemory,
    extent: vk::Extent3D,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
    tiling: vk::ImageTiling,
    properties: vk::MemoryPropertyFlags,
    is_swapchain_image: bool,
}

impl Image {
    pub(crate) fn new_for_swapchain(
        device: &Rc<Device>,
        image: vk::Image,
        extent: vk::Extent2D,
        format: vk::Format,
    ) -> TvResult<Self> {
        let extent = vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        };

        let usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_SRC;
        let tiling = vk::ImageTiling::OPTIMAL;
        let properties = vk::MemoryPropertyFlags::DEVICE_LOCAL;

        Ok(Self {
            device: device.clone(),
            image,
            memory: vk::DeviceMemory::null(),
            extent,
            format,
            usage,
            tiling,
            properties,
            is_swapchain_image: true,
        })
    }

    pub(crate) fn new(
        device: &Rc<Device>,
        extent: vk::Extent3D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
        tiling: vk::ImageTiling,
        properties: vk::MemoryPropertyFlags,
    ) -> TvResult<Self> {
        todo!("Image::new")
        // let image_create_info = vk::ImageCreateInfo::builder()
        //     .image_type(vk::ImageType::TYPE_2D)
        //     .extent(extent)
        //     .mip_levels(1)
        //     .array_layers(1)
        //     .format(format)
        //     .tiling(tiling)
        //     .initial_layout(vk::ImageLayout::UNDEFINED)
        //     .usage(usage)
        //     .sharing_mode(vk::SharingMode::EXCLUSIVE)
        //     .samples(vk::SampleCountFlags::TYPE_1)
        //     .build();

        // let image = unsafe { device.handle().create_image(&image_create_info, None)? };

        // let memory_requirements = unsafe { device.handle().get_image_memory_requirements(image) };

        // let memory_type_index = device
        //     .handle()
        //     .find_memory_type_index(memory_requirements.memory_type_bits, properties)?;

        // let memory_allocate_info = vk::MemoryAllocateInfo::builder()
        //     .allocation_size(memory_requirements.size)
        //     .memory_type_index(memory_type_index)
        //     .build();

        // let memory = unsafe {
        //     device
        //         .handle()
        //         .allocate_memory(&memory_allocate_info, None)?
        // };

        // unsafe { device.handle().bind_image_memory(image, memory, 0)? };

        // Ok(Self {
        //     device: device.clone(),
        //     image,
        //     memory,
        //     extent,
        //     format,
        //     usage,
        //     tiling,
        //     properties,
        // })
    }

    pub(crate) fn handle(&self) -> vk::Image {
        self.image
    }

    pub(crate) fn extent(&self) -> vk::Extent3D {
        self.extent
    }

    pub(crate) fn format(&self) -> vk::Format {
        self.format
    }

    pub(crate) fn usage(&self) -> vk::ImageUsageFlags {
        self.usage
    }

    pub(crate) fn tiling(&self) -> vk::ImageTiling {
        self.tiling
    }

    pub(crate) fn properties(&self) -> vk::MemoryPropertyFlags {
        self.properties
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        if self.is_swapchain_image {
            return;
        }

        unsafe {
            self.device.handle().device_wait_idle().unwrap();
        }
        unsafe {
            self.device.handle().free_memory(self.memory, None);
        }
        unsafe {
            self.device.handle().destroy_image(self.image, None);
        }
    }
}
