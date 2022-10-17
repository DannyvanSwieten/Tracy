pub mod application;
pub mod image_renderer;
pub mod instancer;
pub mod load_scene;
pub mod project;
pub mod scene_graph;
pub mod schema;
pub mod server;
pub mod simple_shapes;

use ash::extensions::{ext::DebugUtils, khr::Surface};
use load_scene::load_scene_gltf;
use nalgebra_glm::{vec3, Mat4};
use renderer::{
    cpu_resource_cache::Resources, gpu_path_tracer::Renderer, gpu_resource_cache::GpuResourceCache,
    material_resource::Material,
};

use vk_utils::vulkan::Vulkan;

use futures::lock::Mutex;
use std::{rc::Rc, sync::Arc};

type ServerContext = Arc<Mutex<application::Model>>;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mode = if args.len() == 5 {
        args[1].clone()
    } else {
        panic!("Invalid arguments")
    };

    if mode == "--file".to_string() {
        let vulkan = Vulkan::new(
            "tracey renderer",
            &[std::ffi::CString::new("VK_LAYER_KHRONOS_validation")
                .expect("String Creation Failed")],
            &[
                DebugUtils::name(),
                Surface::name(),
                vk_utils::vulkan::surface_extension_name(),
            ],
        );
        let gpu = &vulkan.hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
        let context = if cfg!(unix) {
            Rc::new(Renderer::create_suitable_device_mac(gpu))
        } else {
            Rc::new(Renderer::create_suitable_device_windows(gpu))
        };

        println!("{}", args[2]);
        let image_width = args[3].parse::<u32>().expect("Invalid width argument");
        let image_height = args[4].parse::<u32>().expect("Invalid height argument");
        let mut renderer = Renderer::new(context.clone(), image_width, image_height);
        renderer.set_camera_position(0f32, 100f32, 5f32);
        renderer.set_camera_target(10.0, 120f32, 0f32);
        let mut cpu_cache = Resources::default();
        cpu_cache.set_default_material(Material::new(&nalgebra_glm::Vec4::new(1.0, 1.0, 1.0, 1.0)));
        let mut gpu_cache = GpuResourceCache::default();

        let mut scenes = load_scene_gltf(&args[2], &mut cpu_cache).unwrap();
        let gpu_scene = scenes[0].build(
            &mut gpu_cache,
            &cpu_cache,
            Mat4::new_nonuniform_scaling(&vec3(1.0, 1.0, 1.0)),
            renderer.device.clone(),
            &renderer.rtx,
            renderer.queue(),
        );
        let frame = renderer.build_frame(&gpu_scene);

        for _ in 0..64 {
            renderer.render_frame(&frame, 4);
        }

        let buffer = renderer.download_image();
        let data = buffer.copy_data::<u8>();
        image::save_buffer(
            "Camera_1.png",
            &data,
            image_width,
            image_height,
            image::ColorType::Rgba8,
        )
        .expect("Image Write failed");
    }
}
