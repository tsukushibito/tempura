use std::any::Any;
use std::rc::Rc;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tmp_vk_renderer::{vulkan::Device, Renderer, Window as TmpWindow};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

struct WinitWindow {
    pub window: Window,
}

unsafe impl HasRawDisplayHandle for WinitWindow {
    fn raw_display_handle(&self) -> raw_window_handle::RawDisplayHandle {
        self.window.raw_display_handle()
    }
}

unsafe impl HasRawWindowHandle for WinitWindow {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        self.window.raw_window_handle()
    }
}

impl TmpWindow for WinitWindow {
    fn window_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn main() {
    println!("render example.");
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Tempura Example")
        .with_inner_size(LogicalSize::new(1080.0, 720.0))
        .build(&event_loop)
        .unwrap();

    let winit_window = Rc::new(WinitWindow { window });

    let device = Rc::new(Device::new(&winit_window).unwrap());
    let renderer = Rc::new(Renderer::new(&device, &winit_window).unwrap());

    event_loop.run_return(|event, _, control_flow| {
        control_flow.set_wait();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == winit_window.window.id() => control_flow.set_exit(),
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
            } if window_id == winit_window.window.id() => {
                println!("window resized. size: {:?}", size);
                renderer.recreate_swapchain().unwrap();
            }
            Event::MainEventsCleared => {
                renderer.render().expect("render error");
            }
            _ => (),
        }
    });
    println!("exit.");
}