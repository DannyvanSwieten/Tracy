use std::rc::Rc;

use ash::extensions::ext::DebugUtils;

use cgmath::vec3;
use image::EncodableLayout;
use renderer::{
    camera::Camera,
    ctx::Ctx,
    image_resource::TextureImageData,
    math::{Vec2, Vec3, Vec4},
    mesh_resource::MeshResource,
    scene::Scene,
    vk::Format,
};
use vk_utils::vulkan::Vulkan;

pub fn create_cube() -> MeshResource {
    let vertices = vec![
        vec3(-0.5, -0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(-0.5, 0.5, -0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(-0.5, -0.5, 0.5),
        vec3(0.5, -0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(-0.5, 0.5, 0.5),
        vec3(-0.5, -0.5, 0.5),
        vec3(-0.5, 0.5, 0.5),
        vec3(-0.5, 0.5, -0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(-0.5, -0.5, 0.5),
        vec3(-0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, -0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(0.5, -0.5, -0.5),
        vec3(0.5, -0.5, 0.5),
        vec3(0.5, -0.5, 0.5),
        vec3(-0.5, -0.5, 0.5),
        vec3(-0.5, -0.5, -0.5),
        vec3(-0.5, 0.5, -0.5),
        vec3(0.5, 0.5, -0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(0.5, 0.5, 0.5),
        vec3(-0.5, 0.5, 0.5),
        vec3(-0.5, 0.5, -0.5),
    ];

    let normals: Vec<Vec3> = vec![
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, -1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(0.0, 0.0, 1.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(-1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(1.0, 0.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, -1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
        vec3(0.0, 1.0, 0.0),
    ];

    let tangents = (0..vertices.len())
        .map(|_| Vec3::new(0.0, 0.0, 0.0))
        .collect();

    let tex_coords = vertices
        .iter()
        .map(|v| {
            let s = v.x + 0.5;
            let t = v.y + 0.5;
            Vec2::new(s, t)
        })
        .collect();

    let indices = (0..vertices.len()).map(|i| i as u32).collect();

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
        Rc::new(Ctx::create_suitable_device_mac(gpu))
    } else {
        Rc::new(Ctx::create_suitable_device_windows(gpu))
    };

    let image_width = 1280;
    let image_height = 720;
    let mut ctx = Ctx::new(device, 1);
    let mut framebuffer = ctx.create_framebuffer(image_width, image_height);
    let mut scene = Scene::new();
    let mut camera = Camera::new(65.0, 0.01, 1000.0);
    camera.translate(vec3(0.0, 0.0, -12.0));
    scene.set_camera(camera);
    let cube = create_cube();
    let cube_mesh = ctx.create_mesh(&cube);
    let cube_instance = ctx.create_instance(cube_mesh);
    let material_handle = ctx.create_material();
    if let Some(material) = ctx.material_mut(material_handle) {
        material.base_color = Vec4::new(1.0, 1.0, 1.0, 1.0);
        material.roughness = 0.25;
    }
    if let Some(instance) = ctx.instance_mut(cube_instance) {
        instance
            .scale(&vec3(3.0, 3.0, 3.0))
            .rotate(&vec3(100.5, 20.5, 100.5));

        instance.set_material(material_handle);
    }

    scene.add_instance(cube_instance);

    let floor_instance = ctx.create_instance(cube_mesh);
    if let Some(instance) = ctx.instance_mut(floor_instance) {
        instance
            .scale(&vec3(1000.0, 0.1, 1000.0))
            .translate(&vec3(0.0, -15.0, 0.0));
    }

    scene.add_instance(floor_instance);

    let cwd = std::env::current_dir().expect("No working directory found");
    let skybox_path = cwd.join("assets/hdr/skybox.exr");
    let image = image::open(skybox_path).expect("Unable to load skybox image");
    let skybox = ctx.create_skybox(&TextureImageData::new(
        Format::R32G32B32A32_SFLOAT,
        image.width(),
        image.height(),
        image.to_rgba32f().as_bytes(),
    ));
    scene.set_skybox(skybox);
    let frame = ctx.build_frame_resources(&framebuffer, &scene);
    ctx.render_frame(&mut framebuffer, &frame, 120, 4);
    let image_data = framebuffer.download_output();
    image::save_buffer(
        "Skybox.png",
        &image_data,
        image_width,
        image_height,
        image::ColorType::Rgba8,
    )
    .expect("Image Write failed");
}
