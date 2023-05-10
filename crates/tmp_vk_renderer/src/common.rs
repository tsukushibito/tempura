use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

pub type TmpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub trait Window: HasRawDisplayHandle + HasRawWindowHandle + std::any::Any {
    fn window_size(&self) -> (u32, u32);
    fn as_any(&self) -> &dyn std::any::Any;
}
