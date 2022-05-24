use ash::vk::{Format, Image, ImageLayout};

pub struct SwapchainImage {
    handle: Image,
    layout: ImageLayout,
    format: Format,
    width: u32,
    height: u32,
}
