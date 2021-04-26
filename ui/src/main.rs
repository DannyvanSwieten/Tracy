pub mod application;
pub mod canvas;
pub mod node;
pub mod swapchain;
pub mod ui_window;
pub mod user_interface;
pub mod widget;
pub mod window;

use application::{Application, ApplicationDelegate};
use ui_window::{UIWindow, WindowDelegate};
use user_interface::UIDelegate;

use winit::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

use std::collections::HashMap;

struct AppState {}

struct Delegate<AppState> {
    windows: HashMap<WindowId, Window>,
    ui_windows: HashMap<WindowId, Box<dyn WindowDelegate<AppState>>>,
}

impl Delegate<AppState> {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            ui_windows: HashMap::new(),
        }
    }
}

struct MyUIDelegate {}
impl<AppState: 'static> UIDelegate<AppState> for MyUIDelegate {
    fn build(&self, _: &str, _: &AppState) -> node::Node<AppState> {
        node::Node::new("body")
            .with_widget(widget::Container::new())
            .with_padding(25.)
            .with_child(
                node::Node::new("div")
                    .with_widget(widget::Stack::new(widget::Orientation::Horizontal))
                    .with_relative_max_constraints(None, Some(33.))
                    .with_rebuild_callback(|_| {
                        Some(std::vec![
                            node::Node::new("btn").with_widget(widget::Button::new("Label 1")),
                            node::Node::new("btn").with_widget(widget::Button::new("Label 2")),
                            node::Node::new("btn").with_widget(widget::Button::new("Label 3")),
                            node::Node::new("btn").with_widget(widget::Button::new("Label 4")),
                        ])
                    })
                    .with_padding(25.)
                    .with_spacing(5.),
            )
    }
}

impl<AppState: 'static> ApplicationDelegate<AppState> for Delegate<AppState> {
    fn application_will_start(
        &mut self,
        app: &Application<AppState>,
        state: &mut AppState,
        target: &EventLoopWindowTarget<()>,
    ) {
        let window = WindowBuilder::new()
            .with_title("First Window!")
            .with_inner_size(LogicalSize::new(1200, 800))
            .build(&target)
            .unwrap();

        let ui = match UIWindow::<AppState>::new(app, state, &window, Box::new(MyUIDelegate {})) {
            Ok(ui_window) => Box::new(ui_window),
            Err(message) => panic!("{}", message),
        };

        self.ui_windows.insert(window.id(), ui);

        self.windows.insert(window.id(), window);
    }

    fn application_updated(
        &mut self,
        app: &Application<AppState>,
        state: &mut AppState,
    ) -> winit::event_loop::ControlFlow {
        for (_, delegate) in self.ui_windows.iter_mut() {
            delegate.draw(app, state)
        }
        winit::event_loop::ControlFlow::Poll
    }

    fn window_resized(
        &mut self,
        _: &Application<AppState>,
        state: &mut AppState,
        id: &winit::window::WindowId,
        size: &winit::dpi::PhysicalSize<u32>,
    ) -> winit::event_loop::ControlFlow {
        if let Some(window) = self.ui_windows.get_mut(id) {
            window.resized(state, size)
        }

        winit::event_loop::ControlFlow::Wait
    }

    fn window_requested_redraw(
        &mut self,
        app: &Application<AppState>,
        state: &AppState,
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
        self.windows.remove(id);
        if self.windows.is_empty() {
            winit::event_loop::ControlFlow::Exit
        } else {
            winit::event_loop::ControlFlow::Wait
        }
    }

    fn mouse_moved(
        &mut self,
        state: &mut AppState,
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
        state: &mut AppState,
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
        state: &mut AppState,
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
        state: &mut AppState,
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
    let app: Application<AppState> = Application::new("My Application");
    app.run(Box::new(Delegate::new()), AppState {});
}
