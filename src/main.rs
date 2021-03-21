pub mod application;
use application::{Application, ApplicationDelegate};
use std::collections::HashMap;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder, WindowId};

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
            .with_inner_size(LogicalSize::new(512.0, 512.0))
            .build(&target)
            .unwrap();

        self.windows.insert(window.id(), window);
    }
    fn application_will_quit(&mut self, _: &EventLoopWindowTarget<()>) {}
}

fn main() {
    let app = Application::new("My Application", Box::new(Delegate::new()));
    app.run();
}
