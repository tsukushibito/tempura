pub trait WindowSizeProvider {
    fn window_size(&self) -> (u32, u32);
}
