pub mod application;
pub mod canvas;
pub mod node;
pub mod ui_window;
pub mod user_interface;
pub mod widget;
pub mod window;

use ui_window::*;

use application::{Application, ApplicationDelegate};

use winit::{
    dpi::LogicalSize,
    event_loop::EventLoopWindowTarget,
    window::{Window, WindowBuilder, WindowId},
};

use std::collections::HashMap;

struct AppState {}

struct Delegate {
    windows: HashMap<WindowId, Window>,
}

impl Delegate {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
        }
    }
}

impl<AppState: 'static> ApplicationDelegate<AppState> for Delegate {
    fn application_will_start(
        &mut self,
        app: &mut Application<AppState>,
        state: &mut AppState,
        target: &EventLoopWindowTarget<()>,
    ) {
        let window = WindowBuilder::new()
            .with_title("First Window!")
            .with_inner_size(LogicalSize::new(1200, 800))
            .build(&target)
            .unwrap();

        let window2 = WindowBuilder::new()
            .with_title("Second Window!")
            .with_inner_size(LogicalSize::new(400, 400))
            .build(&target)
            .unwrap();

        let ui_window = UIWindow::new(app, &window);
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
    let state = AppState {};
    let app: Application<AppState> = Application::new("My Application", state);
    app.run(Delegate::new());
}
