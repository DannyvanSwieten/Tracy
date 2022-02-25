pub mod application;
pub mod load_scene;
pub mod schema;
pub mod server;
use load_scene::load_scene;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use winit_blit::*;

use ash::extensions::ext::DebugUtils;
use renderer::renderer::Renderer;
use vk_utils::vulkan::Vulkan;

use futures::lock::Mutex;
use std::sync::Arc;

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
    let mut mode = String::new();
    let arg_count = args.len();
    if arg_count == 5 {
        mode = args[1].clone();
    } else {
        panic!("Invalid arguments")
    }

    println!("{}", args[2]);
    let width = args[3].parse::<u32>().expect("Invalid width argument");
    let height = args[4].parse::<u32>().expect("Invalid height argument");

    if mode == "--file" {
        let scene = load_scene(&args[2]).unwrap();
        let mut renderer = Renderer::new(&context, width, height);
        renderer.build(&context, &scene);
        for _ in 0..1024 {
            renderer.render(1, &context);
        }
        let buffer = renderer.download_image(&context);
        let data = buffer.copy_data::<u8>();
        image::save_buffer("output.png", &data, width, height, image::ColorType::Rgba8)
            .expect("Image Write failed");
    } else if mode == "--server" {
        // set up server
        let server = application::ServerApplication::new(context, &args[2]);

        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title("Renderer Output")
            .with_inner_size(winit::dpi::PhysicalSize::new(1280, 720))
            .build(&event_loop)
            .unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::RedrawRequested(window_id) => {
                if window_id == window.id() {
                    let (width, height): (u32, u32) = window.inner_size().into();

                    if let Some(mut model) = server.model.try_lock() {
                        let data = model.download_image();

                        let mut pixel_buffer =
                            PixelBufferTyped::<NativeFormat>::new_supported(width, height, &window);

                        for (i, row) in pixel_buffer.rows_mut().enumerate() {
                            for (j, pixel) in row.into_iter().enumerate() {
                                let index = (i * 1270 + j) * 4;
                                let value = &data[index..index + 3];
                                *pixel = NativeFormat::from_rgb(value[0], value[1], value[2]);
                            }
                        }

                        pixel_buffer.blit(&window).unwrap();
                    }
                }
            }
            _ => {
                if let Some(model) = server.model.try_lock() {
                    if model.has_new_frame {
                        window.request_redraw();
                    }
                }
                *control_flow = ControlFlow::WaitUntil(
                    std::time::Instant::now() + std::time::Duration::from_millis(60),
                )
            }
        });
    }
}
