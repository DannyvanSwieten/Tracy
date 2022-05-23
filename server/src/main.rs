pub mod application;
pub mod image_resource;
pub mod instancer;
pub mod load_scene;
pub mod material_resource;
pub mod mesh_resource;
pub mod parameter;
pub mod project;
pub mod resource;
pub mod resources;
pub mod scene_graph;
pub mod schema;
pub mod server;
pub mod simple_shapes;

use ash_window::enumerate_required_extensions;
use load_scene::load_scene_gltf;
use nalgebra_glm::{vec3, Mat4};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    vk::SurfaceKHR,
};
use renderer::gpu_path_tracer::Renderer;
use vk_utils::{swapchain::Swapchain, vulkan::Vulkan};

use futures::lock::Mutex;
use std::{rc::Rc, sync::Arc};

use crate::resources::{GpuResourceCache, Resources};

type ServerContext = Arc<Mutex<application::Model>>;
fn main() {
    let vulkan = Vulkan::new(
        "tracey renderer",
        &[std::ffi::CString::new("VK_LAYER_KHRONOS_validation").expect("String Creation Failed")],
        &[
            DebugUtils::name(),
            Surface::name(),
            vk_utils::vulkan::surface_extension_name(),
        ],
    );
    let gpu = &vulkan.hardware_devices_with_queue_support(ash::vk::QueueFlags::GRAPHICS)[0];
    let extensions = gpu.device_extensions();
    for extension in extensions {
        let v = extension.extension_name.iter().map(|i| *i as u8).collect();
        println!("{}", String::from_utf8(v).unwrap());
    }
    let context = if cfg!(unix) {
        Rc::new(Renderer::create_suitable_device_mac(gpu))
    } else {
        Rc::new(Renderer::create_suitable_device_windows(gpu))
    };

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
        let mut renderer = Renderer::new(context.clone(), image_width, image_height);
        let mut cpu_cache = Resources::default();
        let mut gpu_cache = GpuResourceCache::default();

        let mut scenes = load_scene_gltf(&args[2], &mut cpu_cache).unwrap();
        let gpu_scene = scenes[0].build(
            &mut gpu_cache,
            &cpu_cache,
            Mat4::new_nonuniform_scaling(&vec3(1.0, 1.0, 1.0)),
            renderer.device.clone(),
            &renderer.rtx,
        );
        let frame = renderer.build_frame(&gpu_scene);

        renderer.render_frame(&frame, 16);

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
    } else if mode == "--server".to_string() {
        // set up server
        // let server = application::ServerApplication::new(
        //     context.clone(),
        //     &args[2],
        //     image_width,
        //     image_height,
        // );

        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Renderer Output")
            .with_inner_size(winit::dpi::PhysicalSize::new(image_width, image_height))
            .with_min_inner_size(winit::dpi::PhysicalSize::new(image_width, image_height))
            .with_resizable(false)
            .build(&event_loop)
            .unwrap();

        let surface = unsafe {
            ash_window::create_surface(&vulkan.library(), &vulkan.vk_instance(), &window, None)
        };

        let mut swapchain = match surface {
            Ok(s) => Some(Swapchain::new(
                context.clone(),
                s,
                None,
                0,
                image_width,
                image_height,
            )),
            Err(error) => {
                println!("{}", error);
                None
            }
        };

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(window_id) => {
                if window_id == window.id() {
                    let (width, height): (u32, u32) = window.inner_size().into();
                    match swapchain.as_mut().unwrap().next_frame_buffer() {
                        Ok((success, frame_index, framebuffer, semaphore)) => (),
                        Err(err) => println!("{}", err),
                    }

                    // if let Some(mut model) = server.model.try_lock() {
                    //     let data = model.download_image();

                    //     let mut pixel_buffer =
                    //         PixelBufferTyped::<NativeFormat>::new_supported(width, height, &window);

                    //     for (i, row) in pixel_buffer.rows_mut().enumerate() {
                    //         let w = row.len();
                    //         for (j, pixel) in row.into_iter().enumerate() {
                    //             let index = (i * w + j) * 4;
                    //             let value = &data[index..index + 3];
                    //             *pixel = NativeFormat::from_rgb(value[0], value[1], value[2]);
                    //         }
                    //     }

                    //     pixel_buffer.blit(&window).unwrap();
                    // }
                }
            }
            _ => {
                // if let Some(model) = server.model.try_lock() {
                //     if model.has_new_frame {
                //         window.request_redraw();
                //     }
                // }
                *control_flow = ControlFlow::WaitUntil(
                    std::time::Instant::now() + std::time::Duration::from_millis(100),
                )
            }
        });
    }
}
