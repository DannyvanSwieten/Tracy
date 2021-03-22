pub mod application;
use application::{Application, ApplicationDelegate};

use winit::{
    window::{Window, WindowBuilder, WindowId},
    dpi::{LogicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
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
    fn application_will_quit(&mut self, _: &EventLoopWindowTarget<()>) {
        println!("Application will quit")
    }
    fn application_received_window_event(&mut self, event: &winit::event::Event<()>) -> ControlFlow{
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id: _,
            } => return winit::event_loop::ControlFlow::Exit,
            _ => return winit::event_loop::ControlFlow::Wait,
        }
    }
}

fn main() {
    let app = Application::new("My Application", Box::new(Delegate::new()));
    app.run();
}
