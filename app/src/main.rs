pub mod load_scene;
use load_scene::load_scene;

use ash::extensions::ext::DebugUtils;
use renderer::renderer::Renderer;
use vk_utils::vulkan::Vulkan;

fn main() {
    let vulkan = Vulkan::new(
        "tracey renderer",
        &[std::ffi::CString::new("VK_LAYER_KHRONOS_validation").expect("String Creation Failed")],
        &[DebugUtils::name()],
    );
    let gpu = &vulkan.hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
    let context = Renderer::create_suitable_device(gpu);
    let mut renderer = Renderer::new(&context, 1280, 720);

    let args: Vec<String> = std::env::args().collect();
    let scene = load_scene(&args[1]).unwrap();

    renderer.build(&context, &scene);
    renderer.render(&context);
    let buffer = renderer.download_image(&context);
    let data = buffer.copy_data::<u8>();
    image::save_buffer("output.png", &data, 1280, 720, image::ColorType::Rgba8)
        .expect("Image Write failed");
}
