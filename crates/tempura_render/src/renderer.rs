use std::rc::Rc;

use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub trait Renderer {
    type Swapchain;
    type Shader;
    type Material;

    fn render(&self, swapchain: &Self::Swapchain);

    fn create_swapchain(
        self: &Rc<Self>,
        display_handle: &RawDisplayHandle,
        window_handle: &RawWindowHandle,
        window_size: &Rc<dyn WindowSizeProvider>,
    ) -> Self::Swapchain;

    fn create_shader(
        self: &Rc<Self>,
        vertex_shader_code: &Vec<u8>,
        fragment_shader_code: &Vec<u8>,
    ) -> Self::Shader;

    fn create_material(self: &Rc<Self>, shader: &Rc<Self::Shader>) -> Self::Material;
}

pub trait Swapchain {}

pub trait Shader {}
pub trait RenderTarget {}

pub trait WindowSizeProvider {
    fn window_size(&self) -> (u32, u32);
}

pub trait Material {
    type Shader;
    fn shader(&self) -> Rc<Self::Shader>;
}
