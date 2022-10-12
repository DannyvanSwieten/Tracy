use crate::{application::Application, application_model::ApplicationModel};
use std::path::Path;

pub trait WindowDelegate<Model: ApplicationModel> {
    fn close_button_pressed(&mut self, _state: &mut Model) -> bool {
        true
    }
    fn file_hovered(&mut self, _state: &mut Model, _path: &Path, _x: f32, _y: f32) {}
    fn file_dropped(&mut self, _state: &mut Model, _path: &Path, _x: f32, _y: f32) {}
    fn mouse_moved(&mut self, _state: &mut Model, _x: f32, _y: f32) {}
    fn mouse_dragged(&mut self, _state: &mut Model, _x: f32, _y: f32, _dx: f32, _dy: f32) {}
    fn mouse_down(&mut self, app: &mut Application<Model>, _state: &mut Model, _x: f32, _y: f32) {}
    fn mouse_up(&mut self, app: &mut Application<Model>, _state: &mut Model, _x: f32, _y: f32) {}
    fn resized(
        &mut self,
        _window: &winit::window::Window,
        _app: &Application<Model>,
        _state: &mut Model,
        _width: u32,
        _height: u32,
    ) {
    }
    fn keyboard_event(&mut self, _state: &mut Model, _event: &winit::event::KeyboardInput) {}
    fn character_received(&mut self, _state: &mut Model, _character: char) {}
    fn draw(&mut self, _app: &Application<Model>, _state: &Model) {}

    fn update(&mut self, _state: &mut Model) {}
}
