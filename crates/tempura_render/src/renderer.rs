use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use ash::vk;
use tempura_vulkan::{
    attachments_for_swapchain, CommandBuffer, CommandPool, Device, Fence, Framebuffer, QueueFamily,
    RenderPass, Semaphore, Swapchain, Window,
};

use crate::RenderPassCache;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct FrameData {
    image_available_semaphore: Semaphore,
    render_finished_semaphore: Semaphore,
    in_flight_fence: Fence,
    command_pool: Rc<CommandPool>,
    command_buffer: CommandBuffer,
}

pub struct Renderer<T: Window> {
    device: Rc<Device>,
    swapchain: RefCell<Swapchain>,
    framebuffers: RefCell<Vec<Framebuffer>>,
    render_pass_cache: RenderPassCache,
    window: Rc<T>,
    frame_datas: [FrameData; MAX_FRAMES_IN_FLIGHT],
    render_pass: RefCell<Rc<RenderPass>>,
    current_frame: Cell<usize>,
}

impl<T: Window> Renderer<T> {
    pub fn new(device: &Rc<Device>, window: &Rc<T>) -> Result<Self, Box<dyn std::error::Error>> {
        let swapchain = Swapchain::new(device, window.as_ref())?;

        let frame_datas = core::array::from_fn(|_| {
            let image_available_semaphore = Semaphore::new(device).unwrap();
            let render_finished_semaphore = Semaphore::new(device).unwrap();
            let in_flight_fence = Fence::new(device, true).unwrap();
            let command_pool = Rc::new(CommandPool::new(device, QueueFamily::Graphics).unwrap());
            let command_buffer = command_pool
                .allocate_command_buffers(vk::CommandBufferLevel::PRIMARY, 1)
                .unwrap()
                .remove(0);

            FrameData {
                image_available_semaphore,
                render_finished_semaphore,
                in_flight_fence,
                command_pool,
                command_buffer,
            }
        });

        let render_pass_cache = RenderPassCache::new();
        let attachments = attachments_for_swapchain(&swapchain);
        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build()])
            .build()];
        let (render_pass, _) =
            render_pass_cache.get_or_create(&device, &attachments, &subpasses, &[]);
        let framebuffers = create_framebuffers(&device, &swapchain, &render_pass);

        Ok(Self {
            device: device.clone(),
            swapchain: RefCell::new(swapchain),
            render_pass_cache: RenderPassCache::new(),
            framebuffers: RefCell::new(framebuffers),
            window: window.clone(),
            frame_datas,
            render_pass: RefCell::new(render_pass),
            current_frame: Cell::new(0),
        })
    }

    pub fn render(&self) -> Result<(), Box<dyn std::error::Error>> {
        let frame_data = &self.frame_datas[self.current_frame.get()];
        frame_data.in_flight_fence.wait()?;
        frame_data.in_flight_fence.reset()?;
        let result = self
            .swapchain
            .borrow()
            .acquire_next_image(&frame_data.image_available_semaphore);

        let index = match result {
            Ok((image_index, _)) => image_index,
            Err(e) => {
                let vk_result = e.downcast_ref::<vk::Result>();
                match vk_result {
                    Some(_) => {
                        self.swapchain.replace(
                            Swapchain::new(&self.device, self.window.as_ref())
                                .expect("Failed to create swapchain"),
                        );
                        return Ok(());
                    }
                    None => return Err(e),
                }
            }
        };

        frame_data.command_pool.reset()?;
        let command_buffer = &frame_data.command_buffer;
        command_buffer.begin(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT, None)?;
        command_buffer.begin_render_pass(
            &self.render_pass.borrow(),
            &self.framebuffers.borrow()[index as usize],
            &vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain.borrow().image_extent(),
            },
            &[vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.0, 0.0, 1.0, 1.0],
                },
            }],
            vk::SubpassContents::INLINE,
        );
        command_buffer.end_render_pass();
        command_buffer.end()?;

        self.device.graphics_queue().submit(
            &[command_buffer],
            &[&frame_data.image_available_semaphore],
            &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            &[&frame_data.render_finished_semaphore],
            Some(&frame_data.in_flight_fence),
        )?;

        self.device.present_queue().present(
            &self.swapchain.borrow(),
            index,
            &[&frame_data.render_finished_semaphore],
        )?;

        self.current_frame
            .set((self.current_frame.get() + 1) % MAX_FRAMES_IN_FLIGHT);
        Ok(())
    }

    pub fn recreate_swapchain(&self) -> Result<(), Box<dyn std::error::Error>> {
        unsafe { self.device.handle().device_wait_idle()? };
        self.swapchain.borrow_mut().recreate(self.window.as_ref())?;
        let attachments = attachments_for_swapchain(&self.swapchain.borrow());
        let subpasses = [vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&[vk::AttachmentReference::builder()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build()])
            .build()];
        let (render_pass, _) =
            self.render_pass_cache
                .get_or_create(&self.device, &attachments, &subpasses, &[]);
        self.render_pass.replace(render_pass);
        self.framebuffers.replace(create_framebuffers(
            &self.device,
            &self.swapchain.borrow(),
            &self.render_pass.borrow(),
        ));
        Ok(())
    }
}

fn create_framebuffers(
    device: &Rc<Device>,
    swapchain: &Swapchain,
    render_pass: &Rc<RenderPass>,
) -> Vec<Framebuffer> {
    let mut framebuffers = Vec::new();
    for image_view in swapchain.image_views() {
        let framebuffer = Framebuffer::new(device, render_pass, &image_view, 1).unwrap();
        framebuffers.push(framebuffer);
    }
    framebuffers
}
