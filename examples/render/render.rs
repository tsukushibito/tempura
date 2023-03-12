use std::{mem::swap, rc::Rc};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use tempura_render::{Renderer, WindowSizeProvider};
use tempura_vulkan_render as vulkan;
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn,
    window::{Window, WindowBuilder},
};

struct WinitWindow {
    window: Rc<Window>,
}

impl WindowSizeProvider for WinitWindow {
    fn window_size(&self) -> (u32, u32) {
        (
            self.window.inner_size().width,
            self.window.inner_size().height,
        )
    }
}

fn main() {
    println!("render example.");
    let mut event_loop = EventLoop::new();
    let window: Rc<Window> = Rc::new(
        WindowBuilder::new()
            .with_title("Tempura Example")
            .with_inner_size(LogicalSize::new(1080.0, 720.0))
            .build(&event_loop)
            .unwrap(),
    );

    let device = Rc::new(vulkan::Device::new(&window.raw_display_handle()));
    let renderer = Rc::new(vulkan::Renderer::new(&device));
    let window_size_provider: Rc<dyn WindowSizeProvider> = Rc::new(WinitWindow {
        window: window.clone(),
    });
    let swapchain = vulkan::Swapchain::new(
        &device,
        &window.raw_display_handle(),
        &window.raw_window_handle(),
        &window_size_provider,
    );

    // let vertex_shader_code = include_bytes!("shaders/triangle.vert.spv").to_vec();
    // let fragment_shader_code = include_bytes!("shaders/triangle.frag.spv").to_vec();
    // let shader = Rc::new(renderer.create_shader(&vertex_shader_code, &fragment_shader_code));
    // let _material = Rc::new(renderer.create_material(&shader));

    event_loop.run_return(|event, _, control_flow| {
        control_flow.set_wait();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => control_flow.set_exit(),
            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(_size),
            } if window_id == window.id() => {
                // println!("window resized. size: {:?}", _size)
            }
            Event::MainEventsCleared => {
                //window.request_redraw();
                let rt = swapchain
                    .acquire_next_render_target()
                    .expect("acquire_next_render_target error.");
                renderer.render(&rt);
                swapchain.present();
            }
            _ => (),
        }
    });
    println!("exit.");
}
