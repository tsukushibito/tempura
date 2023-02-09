use std::rc::Rc;

pub trait Renderer {
    type RenderTarget;

    fn render(&self, swapchain: &Self::RenderTarget);
}

pub trait WindowSizeProvider {
    fn window_size(&self) -> (u32, u32);
}

pub trait Swapchain {
    type RenderTarget;
    fn acquire_next_render_target() -> Rc<Self::RenderTarget>;
    fn present();
}
