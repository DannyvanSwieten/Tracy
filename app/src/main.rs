use ash::extensions::ext::DebugUtils;
use renderer::{geometry::Vertex, renderer::Renderer, scene::Scene};
use vk_utils::vulkan::Vulkan;

fn main() {
    let vulkan = Vulkan::new(
        "tracey renderer",
        &[std::ffi::CString::new("VK_LAYER_KHRONOS_validation").expect("String Creation Failed")],
        &[DebugUtils::name()],
    );
    let gpu = &vulkan.hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
    let context = Renderer::create_suitable_device(gpu);
    let mut renderer = Renderer::new(&context, 1920, 1080);

    let vertices = [
        Vertex::new(-1.0, -1.0, 0.0),
        Vertex::new(1.0, -1.0, 0.0),
        Vertex::new(0.0, 1.0, 0.0),
    ];

    let indices = [0, 1, 2];

    let mut scene = Scene::new();
    let geometry_id = scene.add_geometry(&indices, &vertices);
    scene.create_instance(geometry_id);
    renderer.build(&context, &scene);
    renderer.render(&context);
    let buffer = renderer.download_image(&context);
    let data = buffer.copy_data::<u8>();
    image::save_buffer("output.png", &data, 1920, 1080, image::ColorType::Rgba8)
        .expect("Image Write failed");
}
