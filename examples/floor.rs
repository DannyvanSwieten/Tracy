use std::rc::Rc;

use ash::extensions::ext::DebugUtils;

use cgmath::{vec2, vec3};
use renderer::{
    ctx::Ctx, gpu_path_tracer::Renderer, math::Vec4, mesh_resource::MeshResource, scene::Scene,
};
use vk_utils::vulkan::Vulkan;

pub fn create_floor() -> MeshResource {
    let vertices = vec![
        vec3(-1.0, 0.0, 1.0),
        vec3(-1.0, 0.0, 1.0),
        vec3(1.0, 0.0, -1.0),
        vec3(1.0, 0.0, 1.0),
    ];

    let normals = vec![
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
    ];

    let tangents = vec![
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
    ];

    let tex_coords = vec![
        vec2(0.0, 1.0),
        vec2(0.0, 0.0),
        vec2(1.0, 0.0),
        vec2(1.0, 1.0),
    ];

    let indices = vec![0, 1, 2, 0, 2, 3];

    MeshResource {
        indices,
        vertices,
        normals,
        tangents,
        tex_coords,
    }
}

fn main() {
    let vulkan = Vulkan::new(
        "tracey renderer",
        &[std::ffi::CString::new("VK_LAYER_KHRONOS_validation").expect("String Creation Failed")],
        &[DebugUtils::name()],
    );
    let gpu = &vulkan.hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
    let device = if cfg!(unix) {
        Rc::new(Renderer::create_suitable_device_mac(gpu))
    } else {
        Rc::new(Renderer::create_suitable_device_windows(gpu))
    };

    let image_width = 1280;
    let image_height = 720;
    let mut ctx = Ctx::new(device.clone());
    let floor = create_floor();
    let floor_mesh = ctx.create_mesh(&floor);
    let floor_material = ctx.create_material();
    let floor_instance = ctx.create_instance(floor_mesh);
    if let Some(modifier) = ctx.material_mut(floor_material).as_mut() {
        modifier.base_color = Vec4::new(1.0, 0.0, 0.0, 1.0);
    }
    let mut scene = Scene::new();
    scene.add_instance(floor_instance);
    let frame = ctx.build_frame(&scene);
    print!("{}", 0);
    // let mut renderer = Renderer::new(device.clone(), image_width, image_height);
    // renderer.set_camera_position(0.0, 0.0, 5.0);
    // renderer.set_camera_target(0.0, 0.0, 0.0);
    // let mut gpu_scene = Scene::default();
    // let frame = renderer.build_frame(&gpu_scene);

    // for _ in 0..64 {
    //     renderer.render_frame(&frame, 4);
    // }

    // let buffer = renderer.download_image();
    // let data = buffer.copy_data::<u8>();
    // image::save_buffer(
    //     "Camera_1.png",
    //     &data,
    //     image_width,
    //     image_height,
    //     image::ColorType::Rgba8,
    // )
    // .expect("Image Write failed");
}
