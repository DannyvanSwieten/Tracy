use image::save_buffer;
use renderer::geometry::*;
use renderer::renderer::*;
use ui::{
    application::{Application, ApplicationDelegate, WindowRegistry},
    node::Node,
    ui_window::UIWindowDelegate,
    user_interface::UIDelegate,
    widget::*,
    window_event::MouseEventType,
};
use winit::event_loop::EventLoopWindowTarget;

struct MyState {
    count: u32,
}

struct Delegate {
    renderer: Option<Renderer>,
}

struct MyUIDelegate {}
impl UIDelegate<MyState> for MyUIDelegate {
    fn build(&self, _: &str, _: &MyState) -> Node<MyState> {
        Node::new("body")
            .with_widget(Container::new())
            .with_padding(25.)
            .with_child(
                Node::new("div")
                    .with_name("root")
                    .with_widget(Stack::new(Orientation::Horizontal))
                    .with_rebuild_callback(|state| {
                        Some(std::vec![
                            Node::<MyState>::new("btn")
                                .with_widget(Button::new("Up"))
                                .with_event_callback(MouseEventType::MouseUp, |_event, state| {
                                    state.count = state.count + 1;
                                    Action::Layout {
                                        nodes: vec!["root"],
                                    }
                                }),
                            Node::<MyState>::new("btn")
                                .with_widget(Button::new("Reset"))
                                .with_event_callback(MouseEventType::MouseUp, |_event, state| {
                                    state.count = 0;
                                    Action::Layout {
                                        nodes: vec!["root"],
                                    }
                                }),
                            Node::new("btn").with_widget(Label::new(
                                &(String::from("Count: ") + &state.count.to_string())
                            )),
                        ])
                    })
                    .with_padding(25.)
                    .with_spacing(5.),
            )
    }
}

impl ApplicationDelegate<MyState> for Delegate {
    fn application_will_update(
        &mut self,
        app: &Application<MyState>,
        state: &mut MyState,
        window_registry: &mut WindowRegistry<MyState>,
        target: &EventLoopWindowTarget<()>,
    ) {
        if let Some(renderer) = &mut self.renderer {
            renderer.render();
        }
    }
    fn application_will_start(
        &mut self,
        app: &Application<MyState>,
        state: &mut MyState,
        window_registry: &mut WindowRegistry<MyState>,
        target: &EventLoopWindowTarget<()>,
    ) {
        let mut scene = renderer::scene::Scene::new();
        let (document, buffers, _) = gltf::import(
            "C:\\Users\\danny\\Documents\\code\\tracey\\assets\\Cube\\glTF\\Cube.gltf",
        )
        .unwrap();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                let vertices: Vec<Vertex> = reader
                    .read_positions()
                    .unwrap()
                    .map(|vertex_position| {
                        Vertex::new(vertex_position[0], vertex_position[1], vertex_position[2])
                    })
                    .collect();

                let indices: Vec<u32> = if let Some(iter) = reader.read_indices() {
                    iter.into_u32().collect()
                } else {
                    (0..vertices.len() as u32).collect()
                };

                let geometry_id = scene.add_geometry(indices, vertices);
                scene.create_instance(geometry_id);
            }
        }

        let gpu = &app
            .vulkan()
            .hardware_devices_with_queue_support(renderer::vk::QueueFlags::GRAPHICS)[0];
        self.renderer = Some(Renderer::new(&gpu));
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.initialize(1200, 800);
            renderer.build(&scene);
            renderer.set_camera(&glm::vec3(0., 0., 5.), &glm::vec3(0., 0., 0.));
            renderer.render();
            let output = renderer.download_image().copy_data::<u8>();
            save_buffer("image.png", &output, 1200, 800, image::ColorType::Rgba8)
                .expect("Image write failed");
        }

        let window = window_registry.create_window(target, "Application Title", 1000, 200);

        let ui = match UIWindowDelegate::<MyState>::new(
            app,
            state,
            &window,
            Box::new(MyUIDelegate {}),
        ) {
            Ok(ui_window_delegate) => Box::new(ui_window_delegate),
            Err(message) => panic!("{}", message),
        };
        window_registry.register(window, ui);
    }
}

fn main() {
    let app: Application<MyState> = Application::new("My Application");
    app.run(Box::new(Delegate { renderer: None }), MyState { count: 0 });
}
