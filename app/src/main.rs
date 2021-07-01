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

struct Delegate {}

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
    fn application_will_start(
        &mut self,
        app: &Application<MyState>,
        state: &mut MyState,
        window_registry: &mut WindowRegistry<MyState>,
        target: &EventLoopWindowTarget<()>,
    ) {
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

        let gpu = &app
            .vulkan()
            .hardware_devices_with_queue_support(renderer::vk::QueueFlags::GRAPHICS)[0];
        let mut renderer = Renderer::new(&gpu);
        renderer.initialize(1200, 800);
        let vertices = vec![
            Vertex::new(0.0, 1.0, 0.0),
            Vertex::new(1.0, -1.0, 0.0),
            Vertex::new(-1.0, -1.0, 0.0),
        ];
        let indices: Vec<u32> = vec![0, 1, 2];
        let buffer = GeometryBuffer::new_with_data(indices, vertices);
        let views = [GeometryBufferView::new(3, 0, 3, 0)];
        renderer.build(&buffer, &views);
        renderer.set_camera(&glm::vec3(0., 0., -5.), &glm::vec3(0., 0., 0.));
        renderer.render();
        let output = renderer.download_image().copy_data::<u8>();
        save_buffer("image.png", &output, 1200, 800, image::ColorType::Rgba8)
            .expect("Image write failed");

        window_registry.register(window, ui);
    }
}

fn main() {
    let app: Application<MyState> = Application::new("My Application");
    app.run(Box::new(Delegate {}), MyState { count: 0 });
}
