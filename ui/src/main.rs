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
impl<AppState> UIDelegate<AppState> for MyUIDelegate {
    fn build(&self, _: &str, _: &AppState) -> node::Node<AppState> {
        node::Node::new("body").with_widget(widget::Container::new(5.))
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

        let window2 = WindowBuilder::new()
            .with_title("Second Window!")
            .with_inner_size(LogicalSize::new(400, 400))
            .build(&target)
            .unwrap();

        self.windows.insert(window2.id(), window2);
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
        if self.windows.len() == 0 {
            winit::event_loop::ControlFlow::Exit
        } else {
            winit::event_loop::ControlFlow::Wait
        }
    }
}

fn main() {
    let app: Application<AppState> = Application::new("My Application");
    app.run(Box::new(Delegate::new()), AppState {});
}
