pub mod application;
pub mod image_resource;
pub mod instancer;
pub mod load_scene;
pub mod material_resource;
pub mod mesh_resource;
pub mod parameter;
pub mod resource;
pub mod resources;
pub mod scene_graph;
pub mod schema;
pub mod server;

use load_scene::load_scene_gltf;
use nalgebra_glm::{vec3, Mat4x4};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use ash::extensions::ext::DebugUtils;
use renderer::renderer::Renderer;
use vk_utils::vulkan::Vulkan;

use futures::lock::Mutex;
use std::sync::Arc;

use crate::resources::Resources;

type ServerContext = Arc<Mutex<application::Model>>;
fn main() {
    let vulkan = Vulkan::new(
        "tracey renderer",
        &[std::ffi::CString::new("VK_LAYER_KHRONOS_validation").expect("String Creation Failed")],
        &[DebugUtils::name()],
    );
    let gpu = &vulkan.hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
    let context = Renderer::create_suitable_device(gpu);

    let args: Vec<String> = std::env::args().collect();
    let mode = if args.len() == 5 {
        args[1].clone()
    } else {
        panic!("Invalid arguments")
    };

    println!("{}", args[2]);
    let image_width = args[3].parse::<u32>().expect("Invalid width argument");
    let image_height = args[4].parse::<u32>().expect("Invalid height argument");

    if mode == "--file".to_string() {
        let mut renderer = Renderer::new(&context, image_width, image_height);
        let mut resources = Resources::default();

        let scenes = load_scene_gltf(&args[2], &mut resources).unwrap();
        let gpu_scene = scenes[0].build(
            Mat4x4::new_nonuniform_scaling(&vec3(1.0, 1.0, 1.0)),
            &context,
            &renderer.rtx,
        );
        // let frame = renderer.build_frame(&context, gpu_scene);

        // renderer.render_frame(&context, &frame, 16);

        let buffer = renderer.download_image(&context);
        let data = buffer.copy_data::<u8>();
        image::save_buffer(
            "Camera_1.png",
            &data,
            image_width,
            image_height,
            image::ColorType::Rgba8,
        )
        .expect("Image Write failed");
    } else if mode == "--server".to_string() {
        // set up server
        let server =
            application::ServerApplication::new(context, &args[2], image_width, image_height);

        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Renderer Output")
            .with_inner_size(winit::dpi::PhysicalSize::new(image_width, image_height))
            .with_min_inner_size(winit::dpi::PhysicalSize::new(image_width, image_height))
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(window_id) => {
                // if window_id == window.id() {
                //     let (width, height): (u32, u32) = window.inner_size().into();

                //     if let Some(mut model) = server.model.try_lock() {
                //         let data = model.download_image();

                //         let mut pixel_buffer =
                //             PixelBufferTyped::<NativeFormat>::new_supported(width, height, &window);

                //         for (i, row) in pixel_buffer.rows_mut().enumerate() {
                //             let w = row.len();
                //             for (j, pixel) in row.into_iter().enumerate() {
                //                 let index = (i * w + j) * 4;
                //                 let value = &data[index..index + 3];
                //                 *pixel = NativeFormat::from_rgb(value[0], value[1], value[2]);
                //             }
                //         }

                //         pixel_buffer.blit(&window).unwrap();
                //     }
                // }
            }
            _ => {
                if let Some(model) = server.model.try_lock() {
                    if model.has_new_frame {
                        window.request_redraw();
                    }
                }
                *control_flow = ControlFlow::WaitUntil(
                    std::time::Instant::now() + std::time::Duration::from_millis(100),
                )
            }
        });
    }
}
