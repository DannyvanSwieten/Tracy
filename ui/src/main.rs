pub mod application;
pub mod canvas;
pub mod node;
pub mod swapchain;
pub mod ui_window;
pub mod user_interface;
pub mod widget;
pub mod window;

use window::MouseEventType;

use application::{Application, ApplicationDelegate};
use ui_window::{UIWindow, WindowDelegate};
use user_interface::UIDelegate;

use winit::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

use std::collections::HashMap;

struct MyState {
    count: u32,
}

struct Delegate<MyState> {
    windows: HashMap<WindowId, Window>,
    ui_windows: HashMap<WindowId, Box<dyn WindowDelegate<MyState>>>,
}

impl Delegate<MyState> {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            ui_windows: HashMap::new(),
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
                                    println!("Button 1!");
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
        target: &EventLoopWindowTarget<()>,
    ) {
        let window = WindowBuilder::new()
            .with_title("First Window!")
            .with_inner_size(LogicalSize::new(1200, 800))
            .build(&target)
            .unwrap();

        let ui = match UIWindow::<MyState>::new(app, state, &window, Box::new(MyUIDelegate {})) {
            Ok(ui_window) => Box::new(ui_window),
            Err(message) => panic!("{}", message),
        };

        self.ui_windows.insert(window.id(), ui);

        self.windows.insert(window.id(), window);
    }

    fn application_updated(
        &mut self,
        app: &Application<MyState>,
        state: &mut MyState,
    ) -> winit::event_loop::ControlFlow {
        for (_, delegate) in self.ui_windows.iter_mut() {
            delegate.update(app, state);
            delegate.draw(app, state)
        }
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
        if self.windows.is_empty() {
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
