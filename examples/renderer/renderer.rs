use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::rc::Rc;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::WindowBuilder,
};

use tmp_vk_renderer::{VkRenderer, VkSwapchain};

fn main() {
    println!("render example.");
    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Tempura Example")
        .with_inner_size(LogicalSize::new(1080.0, 720.0))
        .build(&event_loop)
        .unwrap();

    let renderer = Rc::new(
        VkRenderer::new(&window.raw_display_handle(), &window.raw_window_handle()).unwrap(),
    );

    let swapchain = VkSwapchain::new(
        &renderer,
        &window.raw_display_handle(),
        &window.raw_window_handle(),
        window.inner_size().width,
        window.inner_size().height,
    )
    .unwrap();

    event_loop.run_return(|event, _, control_flow| {
        control_flow.set_wait();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(size),
            } if window_id == window.id() => {
                println!("window resized. size: {:?}", size);
            }
            Event::RedrawRequested(_) => {
                renderer.render(&swapchain).expect("Failed to render.");
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => (),
        }
    });
    println!("exit.");
}
