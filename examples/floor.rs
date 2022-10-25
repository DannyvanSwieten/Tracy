use std::rc::Rc;

use ash::extensions::ext::DebugUtils;

use cgmath::{vec2, vec3};
use renderer::{
    ctx::Ctx,
    gpu_path_tracer::Renderer,
    math::{Vec3, Vec4},
    mesh_resource::MeshResource,
    scene::Scene,
};
use vk_utils::vulkan::Vulkan;

pub fn create_floor() -> MeshResource {
    let mut vertices = vec![
        vec3(-1.0, -1.0, 5.0),
        vec3(1.0, -1.0, 5.0),
        vec3(0.0, 1.0, 5.0),
    ];

    let normals = vec![
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
    ];

    let tangents = vec![
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
    ];

    let tex_coords = vec![vec2(0.0, 1.0), vec2(0.0, 0.0), vec2(1.0, 0.0)];

    let vertices = vertices
        .iter_mut()
        .map(|vertex| *vertex * 1000.0 - Vec3::new(0.0, 1.0, 0.0))
        .collect();

    let indices = vec![0, 1, 2];

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
    let mut ctx = Ctx::new(device, 1);
    let mut framebuffer = ctx.create_framebuffer(image_width, image_height);
    let floor = create_floor();
    let floor_mesh = ctx.create_mesh(&floor);
    let floor_material = ctx.create_material();
    let floor_instance = ctx.create_instance(floor_mesh);
    if let Some(mat) = ctx.material_mut(floor_material).as_mut() {
        mat.base_color = Vec4::new(1.0, 0.0, 0.0, 1.0);
    }
    let mut scene = Scene::new();
    scene.add_instance(floor_instance);
    let frame = ctx.build_frame_resources(&framebuffer, &scene);
    ctx.render_frame(&mut framebuffer, &frame);
    let image_data = framebuffer.download_output();
    image::save_buffer(
        "Camera_1.png",
        &image_data,
        image_width,
        image_height,
        image::ColorType::Rgba8,
    )
    .expect("Image Write failed");
}
