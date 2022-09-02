pub mod application;
pub mod image_renderer;
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

use load_scene::load_scene_gltf;
use nalgebra_glm::{vec3, Mat4};
use ui::{
    application_model::ApplicationModel,
    button::TextButton,
    flex::{Expanded, Row},
    widget::{Center, Slider, Widget},
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use ash::extensions::{ext::DebugUtils, khr::Surface};
use renderer::gpu_path_tracer::Renderer;
use vk_utils::vulkan::Vulkan;

use futures::lock::Mutex;
use std::{io::Write, rc::Rc, sync::Arc, thread};

use crate::resources::{GpuResourceCache, Resources};

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

        println!("{}", args[2]);
        let image_width = args[3].parse::<u32>().expect("Invalid width argument");
        let image_height = args[4].parse::<u32>().expect("Invalid height argument");
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
            renderer.queue(),
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
    } else if mode == "--ui_application".to_string() {
        let application = ui::application::Application::<State>::new("My App");
        application.run(
            ui::ui_application_delegate::UIApplicationDelegate::new().with_window(
                "Window 1",
                800,
                600,
                UIBuilder {},
            ),
            State {},
        )
    }
}

pub struct State {}
impl ApplicationModel for State {
    type MessageType = i32;

    fn handle_message(&mut self, msg: Self::MessageType) {
        println!("Message handled: {}", msg)
    }
}

pub struct UIBuilder {}

impl ui::user_interface::UIBuilder<State> for UIBuilder {
    fn build(&self, _section: &str, _state: &State) -> Box<dyn Widget<State>> {
        Box::new(Center::new(
            Row::new()
                .with_child(
                    TextButton::new("Button 1", 25f32).on_click(|_, _| println!("Click 1 ")),
                )
                .with_child(
                    TextButton::new("Button 2 With More Text", 25f32)
                        .on_click(|_, _| println!("Click 2")),
                )
                .with_child(
                    TextButton::new("Button 3", 25f32).on_click(|app, _| app.send_message(1)),
                )
                .with_child(Expanded::new(Slider::new("value")).with_height(32f32))
                .with_spacing(3f32),
        ))
    }
}
