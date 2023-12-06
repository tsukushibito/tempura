use libloading;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::rc::Rc;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

use tmp_vk_renderer::VkRenderer;

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

fn main() {
    println!("render example.");
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Tempura Example")
        .with_inner_size(LogicalSize::new(1080.0, 720.0))
        .build(&event_loop)
        .unwrap();

    let winit_window = Rc::new(WinitWindow { window });

    let renderer = Rc::new(
        VkRenderer::new(
            &winit_window.raw_display_handle(),
            &winit_window.raw_window_handle(),
        )
        .unwrap(),
    );

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
            }
            Event::MainEventsCleared => {}
            _ => (),
        }
    });
    println!("exit.");
}
