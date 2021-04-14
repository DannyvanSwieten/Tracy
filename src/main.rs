pub mod application;
pub mod ui_window;

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
    fn application_will_start(&mut self, app: &mut Application, target: &EventLoopWindowTarget<()>) {
        let window = WindowBuilder::new()
            .with_title("First Window!")
            .with_inner_size(LogicalSize::new(1200, 800))
            .build(&target)
            .unwrap();

        self.windows.insert(window.id(), window);

        let window2 = WindowBuilder::new()
        .with_title("Second Window!")
        .with_inner_size(LogicalSize::new(400, 400))
        .build(&target)
        .unwrap();

        self.windows.insert(window2.id(), window2);
    }

    fn close_button_pressed(&mut self, id: &winit::window::WindowId) -> winit::event_loop::ControlFlow{
        self.windows.remove(id);
        if self.windows.len() == 0 {
            winit::event_loop::ControlFlow::Exit
        } else {
            winit::event_loop::ControlFlow::Wait
        }
    }
}

fn main() {
    let app = Application::new("My Application");
    let d = Delegate::new();
    app.run(d);
}
