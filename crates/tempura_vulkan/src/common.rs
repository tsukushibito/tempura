use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Window: HasRawDisplayHandle + HasRawWindowHandle + std::any::Any {
    fn window_size(&self) -> (u32, u32);
    fn as_any(&self) -> &dyn std::any::Any;
}

pub type RcWindow = std::rc::Rc<Box<dyn Window>>;

pub struct QueueFamilyIndices {
    pub graphics_family: u32,
    pub present_family: u32,
}
