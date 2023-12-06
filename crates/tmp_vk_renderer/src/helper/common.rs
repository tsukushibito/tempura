pub type TmpResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct QueueFamilyIndices {
    pub graphics_family: u32,
    pub present_family: u32,
}

pub enum QueueFamily {
    Graphics,
    Present,
}
