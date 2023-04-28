use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub trait Window: HasRawDisplayHandle + HasRawWindowHandle {
    fn window_size(&self) -> (u32, u32);
}

pub struct QueueFamilyIndices {
    pub graphics_family: u32,
    pub present_family: u32,
}
