pub mod application;
use application::{Application, ApplicationDelegate};

use winit::{
    dpi::LogicalSize,
    event_loop::{EventLoopWindowTarget},
    window::{Window, WindowBuilder, WindowId},
};

use std::collections::HashMap;

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

impl ApplicationDelegate for Delegate {
    fn application_will_start(&mut self, target: &EventLoopWindowTarget<()>) {
        let window = WindowBuilder::new()
            .with_title("Window!")
            .with_inner_size(LogicalSize::new(1200, 800))
            .build(&target)
            .unwrap();

        self.windows.insert(window.id(), window);
    }

    fn window_will_close(&mut self, _: &winit::window::WindowId) -> winit::event_loop::ControlFlow{
        winit::event_loop::ControlFlow::Exit
    }
}

fn main() {
    let app = Application::new("My Application", Box::new(Delegate::new()));
    app.run();
}
