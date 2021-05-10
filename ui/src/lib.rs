pub mod application;
pub mod node;
pub mod swapchain;
pub mod ui_window;
pub mod user_interface;
pub mod widget;
pub mod window;
pub mod window_delegate;

use std::collections::HashMap;

use renderer::renderer;

use window::MouseEventType;

use application::{Application, ApplicationDelegate, WindowRegistry};
use ui_window::{UIWindowDelegate};
use user_interface::UIDelegate;
use window_delegate::WindowDelegate;
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

struct MyState {
    count: u32,
}

struct Delegate<MyState>{
    windows: HashMap<WindowId, Window>,
    ui_windows: HashMap<WindowId, Box<dyn WindowDelegate<MyState>>>,
}

impl Delegate<MyState> {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            ui_windows: HashMap::new()
        }
    }
}

struct MyUIDelegate {}
impl UIDelegate<MyState> for MyUIDelegate {
    fn build(&self, _: &str, _: &MyState) -> node::Node<MyState> {
        node::Node::new("body")
            .with_widget(widget::Container::new())
            .with_padding(25.)
            .with_child(
                node::Node::new("div")
                    .with_name("root")
                    .with_widget(widget::Stack::new(widget::Orientation::Horizontal))
                    .with_relative_max_constraints(None, Some(33.))
                    .with_rebuild_callback(|state| {
                        Some(std::vec![
                            node::Node::<MyState>::new("btn")
                                .with_widget(widget::Button::new("Button"))
                                .with_event_callback(MouseEventType::MouseUp, |_event, state| {
                                    state.count = state.count + 1;
                                    widget::Action::Layout {
                                        nodes: vec!["root"],
                                    }
                                }),
                            node::Node::new("btn").with_widget(widget::Label::new(
                                &(String::from("Count: ") + &state.count.to_string())
                            )),
                        ])
                    })
                    .with_padding(25.)
                    .with_spacing(5.),
            )
    }
}

impl ApplicationDelegate<MyState> for Delegate<MyState> {
    fn application_will_start(
        &mut self,
        app: &Application<MyState>,
        state: &mut MyState,
        window_registry: &mut WindowRegistry<MyState>,
        target: &EventLoopWindowTarget<()>,
    ) {

        let window = window_registry.create_window(target, "Yeah buddy", 1200, 800);

        let ui = match UIWindowDelegate::<MyState>::new(app, state, &window, Box::new(MyUIDelegate {})) {
            Ok(ui_window_delegate) => Box::new(ui_window_delegate),
            Err(message) => panic!("{}", message),
        };

        window_registry.register(window, ui);

        // let vertices: [f32; 9] = [0., 1., 0., 1., -1., 0., -1., -1., 0.];
        // let indices: [u32; 3] = [0, 1, 2];

        // let mut renderer = renderer::Renderer::new(
        //     app.vulkan_instance(),
        //     Some((*app.primary_gpu(), app.present_queue_and_index().1 as u32)),
        // );
        // renderer.initialize(1200, 800);
        // renderer.build(&vertices, &indices);
        // renderer.render();
    }

    fn application_updated(
        &mut self,
        app: &Application<MyState>,
        state: &mut MyState,
    ) -> winit::event_loop::ControlFlow {
        // for (_, delegate) in self.ui_windows.iter_mut() {
        //     delegate.update(app, state);
        //     delegate.draw(app, state)
        // }
        winit::event_loop::ControlFlow::Poll
    }

    fn window_resized(
        &mut self,
        app: &Application<MyState>,
        state: &mut MyState,
        id: &winit::window::WindowId,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> winit::event_loop::ControlFlow {
        if let Some(window) = self.ui_windows.get_mut(id) {
            window.resized(self.windows.get(id).unwrap(), app, state, size)
        }

        winit::event_loop::ControlFlow::Wait
    }

    fn window_requested_redraw(
        &mut self,
        app: &Application<MyState>,
        state: &MyState,
        window_id: &WindowId,
    ) -> winit::event_loop::ControlFlow {
        if let Some(delegate) = self.ui_windows.get_mut(window_id) {
            delegate.draw(app, state)
        }

        winit::event_loop::ControlFlow::Wait
    }

    fn close_button_pressed(
        &mut self,
        id: &winit::window::WindowId,
    ) -> winit::event_loop::ControlFlow {
        self.ui_windows.remove(id);
        self.windows.remove(id);
        if false {
            winit::event_loop::ControlFlow::Exit
        } else {
            winit::event_loop::ControlFlow::Wait
        }
    }

    fn mouse_moved(
        &mut self,
        state: &mut MyState,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) -> winit::event_loop::ControlFlow {
        if let Some(window) = self.ui_windows.get_mut(id) {
            window.mouse_moved(state, position)
        }
        winit::event_loop::ControlFlow::Wait
    }

    fn mouse_dragged(
        &mut self,
        state: &mut MyState,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) -> winit::event_loop::ControlFlow {
        if let Some(window) = self.ui_windows.get_mut(id) {
            window.mouse_dragged(state, position)
        }
        winit::event_loop::ControlFlow::Wait
    }

    fn mouse_down(
        &mut self,
        state: &mut MyState,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) -> winit::event_loop::ControlFlow {
        if let Some(window) = self.ui_windows.get_mut(id) {
            window.mouse_down(state, position)
        }
        winit::event_loop::ControlFlow::Wait
    }

    fn mouse_up(
        &mut self,
        state: &mut MyState,
        id: &winit::window::WindowId,
        position: &winit::dpi::PhysicalPosition<f64>,
    ) -> winit::event_loop::ControlFlow {
        if let Some(window) = self.ui_windows.get_mut(id) {
            window.mouse_up(state, position)
        }
        winit::event_loop::ControlFlow::Wait
    }
}

fn main() {
    let app: Application<MyState> = Application::new("My Application");
    app.run(Box::new(Delegate::new()), MyState { count: 0 });
}
