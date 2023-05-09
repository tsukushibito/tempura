use std::rc::Rc;

use ash::vk;

use crate::{Device, TvResult};

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

    // pub fn new(
    //     device: &Rc<Device>,
    //     extent: vk::Extent3D,
    //     format: vk::Format,
    //     usage: vk::ImageUsageFlags,
    //     tiling: vk::ImageTiling,
    //     properties: vk::MemoryPropertyFlags,
    // ) -> TvResult<Self> {
    //     todo!("Image::new")
    // }

    pub fn handle(&self) -> vk::Image {
        self.image
    }

    pub fn extent(&self) -> vk::Extent3D {
        self.extent
    }

    pub fn format(&self) -> vk::Format {
        self.format
    }

    pub fn usage(&self) -> vk::ImageUsageFlags {
        self.usage
    }

    pub fn tiling(&self) -> vk::ImageTiling {
        self.tiling
    }

    pub fn properties(&self) -> vk::MemoryPropertyFlags {
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
