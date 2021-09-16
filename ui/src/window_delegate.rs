use crate::application::Application;
use std::path::PathBuf;

pub trait WindowDelegate<AppState> {
    fn close_button_pressed(&mut self, _state: &mut AppState) -> bool {
        true
    }
    fn file_hovered(&mut self, _state: &mut AppState, _path: &PathBuf, _x: f32, _y: f32) {}
    fn file_dropped(&mut self, _state: &mut AppState, _path: &PathBuf, _x: f32, _y: f32) {}
    fn mouse_moved(&mut self, _state: &mut AppState, _x: f32, _y: f32) {}
    fn mouse_dragged(&mut self, _state: &mut AppState, _x: f32, _y: f32, _dx: f32, _dy: f32) {}
    fn mouse_down(&mut self, _state: &mut AppState, _x: f32, _y: f32) {}
    fn mouse_up(&mut self, _state: &mut AppState, _x: f32, _y: f32) {}
    fn resized(
        &mut self,
        _window: &winit::window::Window,
        _app: &Application<AppState>,
        _state: &mut AppState,
        _width: u32,
        _height: u32,
    ) {
    }
    fn keyboard_event(&mut self, _state: &mut AppState, _event: &winit::event::KeyboardInput) {}
    fn draw(&mut self, _app: &Application<AppState>, _state: &AppState) {}

    fn update(&mut self, _state: &mut AppState) {}
}
